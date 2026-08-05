use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Args;
use tracing::info;

use crate::{
    library::{Library, QueryResult},
    track::{Track, read_tracks_from_plainlist, scan_dir},
};

#[derive(Args)]
pub struct ResolveArgs {
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long)]
    library: PathBuf,

    #[arg(long)]
    portable: bool,
}

pub fn process_resolve(args: ResolveArgs) -> anyhow::Result<()> {
    let path = &args.input;
    info!("Parsing plainlist at {}...", path.display());
    let plainlist_tracks = read_tracks_from_plainlist(path)
        .with_context(|| format!("failed to parse plainlist file '{}'", path.display()))?;
    info!(
        "Plainlist {} contains {} tracks",
        path.display(),
        plainlist_tracks.len()
    );

    let lib_path = args.library;
    info!("Scanning {} for tracks...", lib_path.display());
    let tracks = scan_dir(&lib_path);
    info!("Found {} tracks", tracks.len());
    let library = Library::from_tracks(tracks);

    let out_path = args
        .output
        .map(|s| PathBuf::from(s).with_extension("m3u"))
        .unwrap_or_else(|| path.with_extension("m3u"));

    resolve_plainlist(&plainlist_tracks, &library, &out_path)?;

    Ok(())
}

/// parse all plainlists in the in_path
/// and return the tracklists and the corresponding output paths
fn parse_plainlists(in_path: &Path, out_path: &Path) -> anyhow::Result<Vec<(Vec<Track>, PathBuf)>> {
    todo!()
    // Ok(Vec::new())
}

fn resolve_plainlist(
    plainlist_tracks: &[Track],
    library: &Library,
    out_path: &Path,
) -> anyhow::Result<()> {
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

    info!("Saving playlist at {}...", out_path.display());
    std::fs::write(out_path, content)?;
    Ok(())
}
