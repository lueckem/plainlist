use std::path::{PathBuf, absolute};

use anyhow::Context;
use clap::Args;
use pathdiff::diff_paths;
use tracing::{debug, error, info, warn};

use crate::{
    library::{Library, QueryResult},
    track::{Track, read_tracks_from_plainlist, scan_dir},
    utils::{PlaylistType, collect_inputs_outputs, write_playlist_m3u},
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
    // TODO: make sure all combinations of absolute and relative input/output/library paths
    // are handled correctly

    // make all paths absolute
    let in_path = absolute(args.input)?;
    let out_path = args.output.map(|p| absolute(p)).transpose()?;
    let lib_path = args.library.canonicalize().context("invalid library")?;

    let in_out_paths = collect_inputs_outputs(
        in_path,
        out_path,
        &PlaylistType::Plainlist,
        &PlaylistType::M3U,
    )?;
    if in_out_paths.is_empty() {
        info!("No plainlists found.");
        return Ok(());
    }

    info!("Scanning library: '{}'", lib_path.display());
    let tracks = scan_dir(&lib_path);
    if tracks.is_empty() {
        error!("No tracks found in library");
        return Ok(());
    } else {
        info!("Library scan complete (found {} tracks)", tracks.len());
    }
    let library = Library::from_tracks(tracks);

    let mut k = 1;
    let num_plainlists = in_out_paths.len();
    let mut num_parse_errors = 0;
    let mut num_playlists = 0;
    let mut num_missing = 0;

    for (i, o) in in_out_paths {
        println!();
        info!(
            "Resolving plainlist [{}/{}]: {}",
            k,
            num_plainlists,
            i.file_name().unwrap().display()
        );
        k += 1;

        let plainlist_tracks = match read_tracks_from_plainlist(&i) {
            Ok(x) => x,
            Err(err) => {
                error!("Failed to parse '{}': {:#}", i.display(), err);
                num_parse_errors += 1;
                continue;
            }
        };

        let mut track_paths = resolve_plainlist(&plainlist_tracks, &library);
        let num_resolved = track_paths.len();
        num_missing += plainlist_tracks.len() - num_resolved;
        info!("Matched {}/{} tracks", num_resolved, plainlist_tracks.len());

        if args.portable {
            // `o` always contains the filename, so this unwrap never fails
            let o_dir = o.parent().unwrap();
            track_paths = track_paths
                .into_iter()
                .map(|p| diff_paths(p, o_dir).unwrap())
                .collect();
        }

        write_playlist_m3u(&track_paths, &o)?;
        info!("Created playlist '{}'", o.display());
        num_playlists += 1;
    }

    println!();
    info!("Finished");
    info!("Plainlists:\t\t{num_plainlists}");
    info!("Playlists created:\t{num_playlists}");
    info!("Parse failures:\t\t{num_parse_errors}");
    info!("Total missing tracks:\t{num_missing}");

    Ok(())
}

/// Resolve the `plainlist_tracks` against the `library` and return the paths of the matched tracks.
fn resolve_plainlist(plainlist_tracks: &[Track], library: &Library) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(plainlist_tracks.len());
    for t in plainlist_tracks {
        let track_path = match library.query(&t) {
            QueryResult::NotFound => {
                warn!("Track not found: '{}'", t);
                continue;
            }
            QueryResult::Found(lt) => {
                debug!("Found: '{}' at '{}'", t, lt.path.display());
                &lt.path
            }
            QueryResult::FoundArtistMismatch(lt) => {
                info!(
                    "Artist mismatch: '{}' vs '{}' at '{}'",
                    t,
                    lt.track,
                    lt.path.display()
                );
                &lt.path
            }
            QueryResult::Ambiguous(lts) => {
                warn!("Ambiguous: found {} matches for track '{}'", lts.len(), t);
                &lts[0].path
            }
        };
        paths.push(track_path.clone());
    }

    paths
}
