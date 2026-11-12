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
    ///
    /// Parses one or multiple plainlists, reads your music library, resolves the tracks in the plainlist against the library, and finally generates m3u playlists that can be read by any music player.
    Resolve(ResolveArgs),
    /// Extract plainlists from other formats
    Extract(ExtractArgs),
    /// Append tracks from a music library to a plainlist, optionally filtered by artist or album
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
