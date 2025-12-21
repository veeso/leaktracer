use leaktracer::LeaktracerAllocator;

#[global_allocator]
static ALLOCATOR: LeaktracerAllocator = LeaktracerAllocator::init();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    leaktracer::init_symbol_table(&["examples", "leaktracer", "diff_alloc_location"]);

    let allocated = function_which_allocates();
    println!("After allocation:");
    print_stats()?;
    println!();

    drop(allocated);
    println!("After dropping the allocated memory:");
    print_stats()?;
    println!();

    Ok(())
}

fn print_stats() -> Result<(), Box<dyn std::error::Error>> {
    println!("Total allocated bytes: {}", ALLOCATOR.allocated());
    leaktracer::with_symbol_table(|table| {
        for (name, symbol) in table.iter() {
            println!(
                "Symbol: {name}, Allocated: {}, Count: {}",
                symbol.allocated(),
                symbol.count()
            );
        }
    })?;

    Ok(())
}

fn function_which_allocates() -> Vec<u8> {
    let vec: Vec<u8> = vec![0; 1024]; // Allocate 1kb
    println!("Allocated {} bytes in the allocating function", vec.len());
    vec
}
