use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use tracing::info;

use crate::{
    normalize::{NormalizedAlbum, NormalizedArtist},
    track::{LocalTrack, scan_dir},
};

#[derive(Args)]
pub struct AppendArgs {
    input: PathBuf,

    #[arg(short, long)]
    library: PathBuf,

    #[arg(long)]
    artist: Option<String>,

    #[arg(long)]
    album: Option<String>,
}

pub fn process_append(args: AppendArgs) -> anyhow::Result<()> {
    let lib_path = args.library.canonicalize().context("invalid library")?;

    // TODO: abort if both artist and album are None?

    let norm_artist = args.artist.map(|a| NormalizedArtist::new(&a));
    let norm_album = args.album.map(|a| NormalizedAlbum::new(&a));

    info!("Scanning library: '{}'", lib_path.display());
    let tracks = scan_dir(&lib_path);

    Ok(())
}

// TODO:
fn is_valid_track(
    lt: &LocalTrack,
    norm_artist: &Option<NormalizedArtist>,
    norm_album: &Option<NormalizedAlbum>,
) -> bool {
    if let Some(album) = norm_album {
        if lt.album.is_none() {
            return false;
        }
        let lt_album = lt.album.as_ref().unwrap();
        // if NormalizedAlbum::new(&lt.album.unwrap()) != album {
        //     return false;
        // }
    }
    false
}
