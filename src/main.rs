#![allow(dead_code)]

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::{logging::init_logging, resolve::process_resolve};

mod library;
mod logging;
mod normalize;
mod resolve;
mod track;

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
    library: String,

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

fn main() -> anyhow::Result<()> {
    // initialize
    init_logging();
    let cli = Cli::parse();

    match cli.command {
        Commands::Resolve(args) => process_resolve(args),
        Commands::Extract(_) => Ok(()),
    }
}
