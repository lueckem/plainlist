use std::path::{Path, PathBuf};

use clap::Args;
use tracing::{error, info};

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

    let lib_path = args.library;
    info!("Scanning {} for tracks...", lib_path.display());
    let tracks = scan_dir(&lib_path);
    info!("Found {} tracks", tracks.len());
    let library = Library::from_tracks(tracks);

    for (i, o) in in_out {
        info!("Parsing plainlist at {}...", i.display());
        let plainlist_tracks = match read_tracks_from_plainlist(&i) {
            Ok(x) => x,
            Err(err) => {
                error!(
                    "Failed to parse plainlist file '{}': {:#}",
                    i.display(),
                    err
                );
                continue;
            }
        };

        info!(
            "Plainlist {} contains {} tracks",
            i.display(),
            plainlist_tracks.len()
        );

        resolve_plainlist(&plainlist_tracks, &library, &o)?;
    }

    Ok(())
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
