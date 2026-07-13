use clap::{Args, Parser, Subcommand, ValueEnum};
use tracing::info;

use crate::logging::init_logging;

mod logging;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Resolve(ResolveArgs),
    Extract(ExtractArgs),
}

#[derive(Args)]
struct ResolveArgs {
    input: String,

    #[arg(short)]
    output: Option<String>,

    #[arg(short, long)]
    library: Option<String>,

    #[arg(long)]
    portable: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum ExtractSource {
    M3U,
    // Spotify,
}

#[derive(Args)]
struct ExtractArgs {
    input: String,

    #[arg(value_enum, long)]
    from: ExtractSource,

    #[arg(short)]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize
    init_logging();
    let cli = Cli::parse();

    match cli.command {
        Commands::Resolve(_) => info!("resolve"),
        Commands::Extract(_) => info!("extract"),
    };

    Ok(())
}
