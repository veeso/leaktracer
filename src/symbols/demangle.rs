use backtrace::BacktraceSymbol;

const UNKNOWN: &str = "<unknown>";

const IGNORE_LIST: &[&str] = &[
    "leaktracer::symbols::demangle::get_demangled_symbol",
    "leaktracer::symbols::SymbolTable::alloc",
    "leaktracer::symbols::SymbolTable::dealloc",
    "leaktracer::alloc::LeaktracerAllocator::trace_allocation",
    "leaktracer::alloc::with_symbol_table_mut",
    "leaktracer::alloc::LeaktracerAllocator::trace",
    "leaktracer::alloc::LeaktracerAllocator::alloc",
    "leaktracer::alloc::LeaktracerAllocator::dealloc",
];

/// Get the name of a symbol from the demangled name table.
pub fn get_demangled_symbol(modules: &[&str]) -> &'static str {
    let bt = backtrace::Backtrace::new();
    let Some(caller) = get_symbol_from_backtrace(&bt, modules) else {
        return UNKNOWN;
    };

    symbol_name(caller).unwrap_or(UNKNOWN)
}

/// Get the symbol at a specific frame in the backtrace.
fn get_symbol_from_backtrace<'a>(
    backtrace: &'a backtrace::Backtrace,
    modules: &[&str],
) -> Option<&'a BacktraceSymbol> {
    // we need to find the LAST frame, whose name starts with one of the modules
    let frame = backtrace
        .frames()
        .iter()
        .enumerate()
        .find_map(|(index, frame)| {
            let symbol = frame.symbols().first()?;

            let name = symbol.name().map(|name| format!("{name}"))?;

            // ignore this call
            if IGNORE_LIST.iter().any(|ignore| name.starts_with(*ignore)) {
                return None;
            }

            if modules.iter().any(|module| name.starts_with(*module)) {
                Some(index)
            } else {
                None
            }
        })?;

    backtrace
        .frames()
        .get(frame)
        .and_then(|frame| frame.symbols().first())
}

/// Get the name of a symbol from a [`BacktraceSymbol`].
fn symbol_name(symbol: &BacktraceSymbol) -> Option<&'static str> {
    // get the name of the symbol except the last part `backtrace::b::h3777baf656cd0c35`
    let name_str = symbol.name().map(|name| format!("{name}"))?;

    let name_string = if let Some(pos) = name_str.rfind("::") {
        &name_str[..pos]
    } else {
        &name_str
    };

    // convert to static str
    Some(Box::leak(name_string.to_string().into_boxed_str()))
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_get_demangled_symbol() {
        let symbol = a();
        assert!(symbol.contains("symbols::demangle"));
    }

    fn a() -> &'static str {
        b()
    }

    fn b() -> &'static str {
        c()
    }

    fn c() -> &'static str {
        get_demangled_symbol(&["leaktracer"])
    }
}
