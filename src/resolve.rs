use std::path::{Path, PathBuf};

use clap::Args;
use tracing::{debug, error, info, warn};

use crate::{
    library::{Library, QueryResult},
    track::{Track, read_tracks_from_plainlist, scan_dir},
    utils::collect_inputs_outputs,
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
    let in_out = collect_inputs_outputs(args.input, args.output, "plainlist", "m3u")?;
    if in_out.is_empty() {
        info!("No plainlists found.");
        return Ok(());
    }

    let lib_path = args.library;
    info!("Scanning library: '{}'", lib_path.display());
    let tracks = scan_dir(&lib_path);
    info!("Library scan complete (found {} tracks)", tracks.len());
    let library = Library::from_tracks(tracks);

    let mut k = 1;
    let num_plainlists = in_out.len();
    let mut num_parse_errors = 0;
    let mut num_playlists = 0;
    let mut num_missing = 0;

    for (i, o) in in_out {
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

        let num_resolved = resolve_plainlist(&plainlist_tracks, &library, &o)?;
        num_missing += plainlist_tracks.len() - num_resolved;
        info!("Matched {}/{} tracks", num_resolved, plainlist_tracks.len());
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

fn resolve_plainlist(
    plainlist_tracks: &[Track],
    library: &Library,
    out_path: &Path,
) -> anyhow::Result<usize> {
    let mut num_resolved = 0;
    let mut content = String::new();
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
        content.push_str(&track_path.to_string_lossy().into_owned());
        content.push('\n');
        num_resolved += 1;
    }

    std::fs::write(out_path, content)?;
    Ok(num_resolved)
}
