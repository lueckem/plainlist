use tracing_subscriber::EnvFilter;

pub fn init_logging() {
    if cfg!(debug_assertions) {
        // Debug build
        tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_env_filter(EnvFilter::new("plainlist=debug"))
            .init();
    } else {
        // Release build
        tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_env_filter(EnvFilter::new("plainlist=info"))
            .with_target(false)
            .init();
    }
}
