use tracing::info;

use crate::logging::init_logging;

mod logging;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize
    init_logging();
    info!("hello world!");

    Ok(())
}
