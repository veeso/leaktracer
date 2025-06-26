use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use crate::symbols::SymbolTable;

thread_local! {
    static IN_ALLOC: Cell<bool> = const { Cell::new(false) };
}

/// Initial size of the symbol table.
/// This is used to preallocate the symbol table to avoid reallocations.
const DEFAULT_SYMBOL_TABLE_SIZE: usize = 1024;

static SYMBOL_TABLE: OnceLock<Mutex<SymbolTable>> = OnceLock::new();

/// This module provides a custom Allocator ([`GlobalAlloc`]) that tracks to log the memory allocations and stores the
/// allocation information for each module in the program.
///
/// ## Example
///
/// ```rust
/// use leaktracer::LeaktracerAllocator;
///
/// #[global_allocator]
/// static ALLOCATOR: LeaktracerAllocator = LeaktracerAllocator::init();
/// ```
pub struct LeaktracerAllocator {
    allocated: AtomicUsize,
}

/// Initializes the leak tracer with a symbol table of the given size.
///
/// Provide the modules to be traced as a slice of static strings.
/// Providing modules is necessary to filter out allocations that are not relevant to the user (such as from [`std`], [`tokio`], etc.).
pub fn init_symbol_table(modules: &'static [&'static str]) {
    SYMBOL_TABLE.get_or_init(|| Mutex::new(SymbolTable::new(DEFAULT_SYMBOL_TABLE_SIZE, modules)));
}

/// Provides a way to access the symbol table in a thread-safe manner.
///
/// Takes a closure `f` that receives a reference to the symbol table and returns a result.
pub fn with_symbol_table<F, R>(
    f: F,
) -> Result<R, PoisonError<std::sync::MutexGuard<'static, SymbolTable>>>
where
    F: FnOnce(&SymbolTable) -> R,
{
    // prevent allocations DURING lock acquisition
    IN_ALLOC.with(|cell| cell.set(true));

    let lock = match SYMBOL_TABLE
        .get()
        .expect("Symbol table not initialized")
        .lock()
    {
        Ok(lock) => lock,
        Err(poisoned) => {
            // free alloc
            IN_ALLOC.with(|cell| cell.set(false));
            // If the lock is poisoned, we return the poisoned error
            return Err(poisoned);
        }
    };

    let res = Ok(f(&lock));

    IN_ALLOC.with(|cell| cell.set(false));

    res
}

/// An enumeration representing the type of allocation operation being traced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AllocOp {
    Alloc,
    Dealloc,
}

impl LeaktracerAllocator {
    /// Creates a new instance of the [`LeaktracerAllocator`].
    pub const fn init() -> Self {
        LeaktracerAllocator {
            allocated: AtomicUsize::new(0),
        }
    }

    /// Returns the total number of bytes allocated by the allocator up to this point.
    pub fn allocated(&self) -> usize {
        self.allocated.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Returns whether the allocation is an external allocation.
    ///
    /// With **external allocation**, we mean that the allocation is not requested by the allocator itself,
    /// but rather by the user of the allocator.
    ///
    /// This is determined by checking if the `IN_ALLOC` thread-local variable is set to `false`.
    fn is_external_allocation(&self) -> bool {
        !IN_ALLOC.get()
    }

    /// Enters the allocation context, marking that an allocation is being made.
    fn enter_alloc(&self) {
        IN_ALLOC.with(|cell| cell.set(true));
    }

    /// Exits the allocation context, marking that the allocation is done.
    fn exit_alloc(&self) {
        IN_ALLOC.with(|cell| cell.set(false));
    }

    /// Traces the allocation, logging the layout of the allocation.
    fn trace_allocation(&self, layout: Layout, table: Option<&mut MutexGuard<SymbolTable>>) {
        // first increment the allocated bytes
        self.allocated
            .fetch_add(layout.size(), std::sync::atomic::Ordering::Relaxed);
        if let Some(table) = table {
            table.alloc(layout.size());
        }
    }

    /// Traces the deallocation, logging the layout of the deallocation.
    fn trace_deallocation(&self, layout: Layout, table: Option<&mut MutexGuard<SymbolTable>>) {
        // first decrement the allocated bytes
        self.allocated
            .fetch_sub(layout.size(), std::sync::atomic::Ordering::Relaxed);
        if let Some(table) = table {
            table.dealloc(layout.size());
        }
    }

    /// Traces the allocation or deallocation operation using the [`Layout`], depending on the [`AllocOp`] type.
    fn trace(&self, layout: Layout, op: AllocOp) {
        // lock symbol table to avoid deadlocks
        let mut lock = SYMBOL_TABLE.get().and_then(|table| table.lock().ok());

        self.enter_alloc();
        match op {
            AllocOp::Alloc => self.trace_allocation(layout, lock.as_mut()),
            AllocOp::Dealloc => self.trace_deallocation(layout, lock.as_mut()),
        }
        self.exit_alloc();
        drop(lock);
    }
}

unsafe impl GlobalAlloc for LeaktracerAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        // if the allocation is not null AND the allocation is external, trace the allocation
        if !ptr.is_null() && self.is_external_allocation() {
            self.trace(layout, AllocOp::Alloc);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !ptr.is_null() && self.is_external_allocation() {
            self.trace(layout, AllocOp::Dealloc);
        }
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_should_tell_if_external_allocation() {
        init_symbol_table(&["leaktracer"]);

        let allocator = LeaktracerAllocator::init();
        assert!(allocator.is_external_allocation());

        IN_ALLOC.with(|cell| cell.set(true));
        assert!(!allocator.is_external_allocation());

        IN_ALLOC.with(|cell| cell.set(false));
        assert!(allocator.is_external_allocation());
    }

    #[test]
    fn test_should_trace_allocations() {
        init_symbol_table(&["leaktracer"]);

        let allocator = LeaktracerAllocator::init();
        let layout = Layout::from_size_align(1024, 8).unwrap();
        allocator.trace(layout, AllocOp::Alloc);
        assert_eq!(allocator.allocated(), 1024);
    }

    #[test]
    fn test_should_trace_deallocations() {
        init_symbol_table(&["leaktracer"]);

        let allocator = LeaktracerAllocator::init();
        let layout = Layout::from_size_align(1024, 8).unwrap();
        allocator.trace(layout, AllocOp::Alloc);
        assert_eq!(allocator.allocated(), 1024);
        allocator.trace(layout, AllocOp::Dealloc);
        assert_eq!(allocator.allocated(), 0);
    }
}
