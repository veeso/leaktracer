mod demangle;

use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;

pub type AllocId = usize;

/// A [`Symbol`] table.
///
/// Each [`Symbol`] is identified by the module name (e.g. `leaktracer::alloc`).
#[derive(Debug)]
pub struct SymbolTable {
    /// The modules that are being traced.
    modules: &'static [&'static str],
    /// Maps symbol names to their corresponding [`Symbol`]s.
    symbols: HashMap<&'static str, Symbol>,
    /// Maps pointers to symbols for quick lookup during deallocation.
    ptr_to_symbol: HashMap<AllocId, &'static str>,
}

impl SymbolTable {
    /// Creates a new [`SymbolTable`] with the given size and modules.
    pub(crate) fn new(size: usize, modules: &'static [&'static str]) -> Self {
        Self {
            modules,
            symbols: HashMap::with_capacity(size),
            ptr_to_symbol: HashMap::with_capacity(size),
        }
    }

    /// Iterates over the [`Symbol`]s in the table, with their names.
    pub fn iter(&self) -> impl Iterator<Item = (&&'static str, &Symbol)> {
        self.symbols.iter()
    }

    /// Gets a [`Symbol`] by its name.
    pub fn get(&self, name: &'static str) -> Option<&Symbol> {
        self.symbols.get(&name)
    }

    /// Increments the allocated bytes for a [`Symbol`].
    pub(crate) fn alloc(&mut self, alloc_id: AllocId, bytes: usize) {
        let name = demangle::get_demangled_symbol(self.modules);

        // If the symbol does not exist, we create it with the given name.
        if !self.symbols.contains_key(&name) {
            self.insert(name);
        }
        // insert the mapping from alloc_id to symbol name
        self.ptr_to_symbol.insert(alloc_id, name);

        let symbol = self.symbols.get_mut(name).expect("Symbol should exist");

        symbol
            .allocated
            .fetch_add(bytes, std::sync::atomic::Ordering::Relaxed);
        symbol
            .count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Decrements the allocated bytes for a [`Symbol`].
    pub(crate) fn dealloc(&mut self, alloc_id: AllocId, bytes: usize) {
        // remove the mapping from alloc_id to symbol name
        let Some(name) = self.ptr_to_symbol.remove(&alloc_id) else {
            return;
        };

        if let Some(symbol) = self.symbols.get_mut(name) {
            // prevent underflow
            symbol
                .allocated
                .fetch_update(
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                    |current| Some(current.saturating_sub(bytes)),
                )
                .ok();
            symbol
                .count
                .fetch_update(
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                    |current| Some(current.saturating_sub(1)),
                )
                .ok();
        }
    }

    /// Inserts a new [`Symbol`] into the table.
    fn insert(&mut self, name: &'static str) {
        self.symbols.insert(
            name,
            Symbol {
                allocated: AtomicUsize::new(0),
                count: AtomicUsize::new(0),
            },
        );
    }
}

/// A slot in the symbol table.
#[derive(Debug)]
pub struct Symbol {
    /// Allocated bytes for this symbol.
    allocated: AtomicUsize,
    /// Allocation count for this symbol.
    count: AtomicUsize,
}

impl Symbol {
    /// Returns the number of bytes allocated for this symbol.
    pub fn allocated(&self) -> usize {
        self.allocated.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns the number of allocations for this symbol.
    pub fn count(&self) -> usize {
        self.count.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_should_allocate_symbol() {
        const ALLOC_ID: AllocId = 42;
        const ALLOC_ID_2: AllocId = 43;

        let mut table = SymbolTable::new(10, &["leaktracer"]);
        table.alloc(ALLOC_ID, 100);
        // get name of the caller
        let name = demangle::get_demangled_symbol(&["leaktracer"]);
        let symbol = table.get(name).expect("Symbol should exist");
        assert_eq!(symbol.allocated(), 100);
        assert_eq!(symbol.count(), 1);
        // check if ptr to symbol mapping exists
        assert_eq!(table.ptr_to_symbol.get(&ALLOC_ID), Some(&name));

        // allocate again
        table.alloc(ALLOC_ID_2, 50);
        let symbol = table.get(name).expect("Symbol should exist");
        assert_eq!(symbol.allocated(), 150);
        assert_eq!(symbol.count(), 2);
        // check if ptr to symbol mapping exists
        assert_eq!(table.ptr_to_symbol.get(&ALLOC_ID_2), Some(&name));

        // deallocate
        table.dealloc(ALLOC_ID, 40);
        let symbol = table.get(name).expect("Symbol should exist");
        assert_eq!(symbol.allocated(), 110);
        assert_eq!(symbol.count(), 1);
        // check if removed
        assert_eq!(table.ptr_to_symbol.get(&ALLOC_ID), None);
    }

    #[test]
    fn test_should_iter_symbol_table() {
        let mut table = SymbolTable::new(10, &["leaktracer"]);

        table.insert("test_symbol_1");
        table.insert("test_symbol_2");
        let symbols: Vec<_> = table.iter().collect();
        assert_eq!(symbols.len(), 2);
        assert!(
            symbols
                .iter()
                .any(|(symbol, _)| **symbol == "test_symbol_1")
        );
        assert!(
            symbols
                .iter()
                .any(|(symbol, _)| **symbol == "test_symbol_2")
        );
    }
}
