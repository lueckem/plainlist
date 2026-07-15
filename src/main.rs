#![allow(dead_code)]

use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};
use tracing::info;

use crate::{logging::init_logging, track::read_tracks_from_plainlist};

mod library;
mod logging;
mod normalize;
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

fn main() -> anyhow::Result<()> {
    // initialize
    init_logging();
    let cli = Cli::parse();

    match cli.command {
        Commands::Resolve(args) => process_resolve(args),
        Commands::Extract(_) => Ok(()),
    }
}

fn process_resolve(args: ResolveArgs) -> anyhow::Result<()> {
    let path = &args.input;
    info!("Parsing plainlist at {}", path);
    let tracks = read_tracks_from_plainlist(path)
        .with_context(|| format!("failed to parse plainlist file '{}'", path))?;
    for t in tracks {
        println!("{t}");
    }

    Ok(())
}
