use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use tracing::info;

use crate::{
    normalize::{NormalizedAlbum, NormalizedArtist},
    track::scan_dir,
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

    info!("Scanning library: '{}'", lib_path.display());
    let mut tracks = scan_dir(&lib_path);

    // filter album
    if let Some(album) = args.album {
        let norm_album = NormalizedAlbum::new(&album);
        tracks = tracks
            .into_iter()
            .filter(|lt| {
                if let Some(lt_album) = &lt.album {
                    NormalizedAlbum::new(lt_album) != norm_album
                } else {
                    false
                }
            })
            .collect();
    }

    // filter artist
    if let Some(artist) = args.artist {
        let norm_artist = NormalizedArtist::new(&artist);
        tracks = tracks
            .into_iter()
            .filter(|lt| {
                lt.track
                    .artists
                    .iter()
                    .map(|a| NormalizedArtist::new(a))
                    .any(|na| na == norm_artist)
            })
            .collect();
    }

    // TODO: sort remaining tracks first by album, then by track_number

    // TODO: append tracks to plainlist

    Ok(())
}
