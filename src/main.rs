#![allow(dead_code)]

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Args, Parser, Subcommand, ValueEnum};
use tracing::info;

use crate::{
    library::{Library, QueryResult},
    logging::init_logging,
    track::{read_tracks_from_plainlist, scan_dir},
};

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

fn process_resolve(args: ResolveArgs) -> anyhow::Result<()> {
    let path = Path::new(&args.input);
    info!("Parsing plainlist at {}...", path.display());
    let plainlist_tracks = read_tracks_from_plainlist(path)
        .with_context(|| format!("failed to parse plainlist file '{}'", path.display()))?;
    info!(
        "Plainlist {} contains {} tracks",
        path.display(),
        plainlist_tracks.len()
    );

    let lib_path = Path::new(args.library.as_str());
    info!("Scanning {} for tracks...", lib_path.display());
    let tracks = scan_dir(lib_path);
    info!("Found {} tracks", tracks.len());
    let library = Library::from_tracks(tracks);

    let mut content = String::new();
    for t in plainlist_tracks {
        let track_path = match library.query(&t) {
            QueryResult::NotFound => {
                info!("NotFound: '{}'", t);
                continue;
            }
            QueryResult::Found(lt) => {
                info!("Found: '{}' at '{}'", t, lt.path.display());
                &lt.path
            }
            QueryResult::FoundArtistMismatch(lt) => {
                info!(
                    "FoundArtistMismatch: '{}' vs '{}' at '{}'",
                    t,
                    lt.track,
                    lt.path.display()
                );
                &lt.path
            }
            QueryResult::Ambiguous(lts) => {
                info!("Ambiguous: found {} matches for track '{}'", lts.len(), t);
                &lts[0].path
            }
        };
        content.push_str(&track_path.to_string_lossy().into_owned());
        content.push('\n');
    }

    let out_path = args
        .output
        .map(|s| PathBuf::from(s).with_extension("m3u"))
        .unwrap_or_else(|| path.with_extension("m3u"));
    info!("Saving playlist at {}...", out_path.display());
    std::fs::write(out_path, content)?;

    Ok(())
}
