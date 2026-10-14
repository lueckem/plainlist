#![allow(dead_code)]

use clap::{Parser, Subcommand};

use crate::{
    extract::{ExtractArgs, process_extract},
    logging::init_logging,
    resolve::{ResolveArgs, process_resolve},
};

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
    Resolve(ResolveArgs),
    Extract(ExtractArgs),
    // TODO: add `append` command, e.g.
    // `plainlist append --library ~/Music --artist Sabaton --album "Primo Victoria" new.plainlist`.
    // - add album and track_number to `Track`
    // - `scan_dir` -> `Vec<LocalTrack>`
    // - filter the artist/album in the vector
    // - sort remaining results first by album, then by track_number
    // - append to plainlist
}

fn main() -> anyhow::Result<()> {
    // initialize
    init_logging();
    let cli = Cli::parse();

    match cli.command {
        Commands::Resolve(args) => process_resolve(args),
        Commands::Extract(args) => process_extract(args),
    }
}
