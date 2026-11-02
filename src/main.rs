#![allow(dead_code)]

use clap::{Parser, Subcommand};

use crate::{
    append::{AppendArgs, process_append},
    extract::{ExtractArgs, process_extract},
    logging::init_logging,
    resolve::{ResolveArgs, process_resolve},
};

mod append;
mod extract;
mod library;
mod logging;
mod normalize;
mod resolve;
mod track;
mod utils;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Resolve plainlists against a music library to generate m3u playlists
    Resolve(ResolveArgs),
    /// Extract plainlists from other formats
    Extract(ExtractArgs),
    /// Append tracks from a music library to a plainlist
    Append(AppendArgs),
}

fn main() -> anyhow::Result<()> {
    // initialize
    init_logging();
    let cli = Cli::parse();

    match cli.command {
        Commands::Resolve(args) => process_resolve(args),
        Commands::Extract(args) => process_extract(args),
        Commands::Append(args) => process_append(args),
    }
}
