use leaktracer::LeaktracerAllocator;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::Layer as _;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt as _;

#[global_allocator]
static ALLOCATOR: LeaktracerAllocator = LeaktracerAllocator::init();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    leaktracer::init_symbol_table(&["examples", "leaktracer", "tracing"]);
    // Initialize tracing
    init_log()?;
    // Log an event
    tracing::info!(
        "Starting the application... Allocated {} bytes",
        ALLOCATOR.allocated()
    );

    // run a task that allocates some memory
    let task_1 = tokio::spawn(task(1024));

    // Run another task that allocates some memory
    let task_2 = tokio::spawn(task(2048));

    // print allocations
    tracing::info!("Total allocated bytes: {}", ALLOCATOR.allocated());
    leaktracer::with_symbol_table(|table| {
        for (name, symbol) in table.iter() {
            tracing::info!(
                "Symbol: {name}, Allocated: {}, Count: {}",
                symbol.allocated(),
                symbol.count()
            );
        }
    })?;

    // Wait for the task to complete
    let _result = task_1.await?;
    let _another_result = task_2.await?;

    let _buff = function_which_allocates();

    leaktracer::with_symbol_table(|table| {
        for (name, symbol) in table.iter() {
            tracing::info!(
                "Symbol: {name}, Allocated: {}, Count: {}",
                symbol.allocated(),
                symbol.count()
            );
        }
    })?;

    // Log completion
    tracing::info!(
        "Application finished successfully. Allocated {} bytes",
        ALLOCATOR.allocated()
    );

    Ok(())
}

fn init_log() -> Result<(), Box<dyn std::error::Error>> {
    let stdout_logger = tracing_subscriber::fmt::layer()
        .compact()
        .with_ansi(true)
        .with_span_events(FmtSpan::CLOSE)
        .with_line_number(true)
        .with_writer(std::io::stdout);

    let registry = tracing_subscriber::registry()
        .with(stdout_logger.with_filter(LevelFilter::from(Level::TRACE)));

    tracing::subscriber::set_global_default(registry)?;

    Ok(())
}

async fn task(sz: usize) -> Vec<u8> {
    let vec: Vec<u8> = vec![0; sz]; // Allocate 1kb
    tracing::info!("Allocated {} bytes in the task", vec.len());
    vec
}

fn function_which_allocates() -> Vec<u8> {
    let vec: Vec<u8> = vec![0; 1024]; // Allocate 1kb
    tracing::info!("Allocated {} bytes in the allocating function", vec.len());
    vec
}
