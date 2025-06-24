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
    // Initialize tracing
    init_log()?;
    // Log an event
    tracing::info!(
        "Starting the application... Allocated {} bytes",
        ALLOCATOR.allocated()
    );

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
