use backtrace::BacktraceSymbol;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    a();

    Ok(())
}

fn a() -> usize {
    b()
}

fn b() -> usize {
    c()
}

fn c() -> usize {
    let bt = backtrace::Backtrace::new();

    let caller = bt.frames().get(1).and_then(|frame| frame.symbols().get(0));
    if let Some(symbol) = caller {
        // base address
        let address = symbol.addr().unwrap_or(std::ptr::null_mut());
        // convert to u64 address
        let address_u64 = address as u64;
        println!("Caller address: {:#x}", address_u64);

        let name = name(&symbol);

        println!("Caller address: {:#x} name: {name}", address_u64);
    }

    42
}

fn name(symbol: &BacktraceSymbol) -> &'static str {
    // get the name of the symbol except the last part `backtrace::b::h3777baf656cd0c35`
    let name_str = symbol
        .name()
        .map(|name| format!("{name}"))
        .unwrap_or("<unknown>".to_string());
    println!("Full symbol name: {name_str}");
    let name_string = if let Some(pos) = name_str.rfind("::") {
        &name_str[..pos]
    } else {
        &name_str
    };

    println!("Symbol name: {name_string}");

    // convert to static str
    Box::leak(name_string.to_string().into_boxed_str())
}
