use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use tracing::info;

use crate::{
    normalize::{NormalizedAlbum, NormalizedArtist},
    track::{LocalTrack, scan_dir},
    utils::write_plainlist,
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
    // Abort if both artist and album are None
    if args.artist.is_none() && args.album.is_none() {
        anyhow::bail!("Either --artist or --album must be specified");
    }

    let mut tracks = scan_and_filter_tracks(&args)?;
    if tracks.is_empty() {
        info!("No matching tracks found");
        return Ok(());
    }
    info!("Found {} matching tracks", tracks.len());

    sort_tracks(&mut tracks);

    let tracks: Vec<_> = tracks.into_iter().map(|lt| lt.track).collect();

    write_plainlist(&tracks, &args.input, true)
        .with_context(|| format!("failed to write to plainlist '{}'", args.input.display()))?;
    info!("Appended matching tracks to '{}'", args.input.display());
    Ok(())
}

/// Scans the music library and filters tracks based on artist or album criteria
fn scan_and_filter_tracks(args: &AppendArgs) -> anyhow::Result<Vec<LocalTrack>> {
    let lib_path = args.library.canonicalize().context("invalid library")?;
    info!("Scanning library: '{}'", lib_path.display());
    let mut tracks = scan_dir(&lib_path);

    // filter album
    if let Some(album) = &args.album {
        let norm_album = NormalizedAlbum::new(album);
        tracks = tracks
            .into_iter()
            .filter(|lt| {
                if let Some(lt_album) = &lt.album {
                    NormalizedAlbum::new(lt_album) == norm_album
                } else {
                    false
                }
            })
            .collect();
    }

    // filter artist
    if let Some(artist) = &args.artist {
        let norm_artist = NormalizedArtist::new(artist);
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

    Ok(tracks)
}

/// Sorts tracks by album first, then by track number
fn sort_tracks(tracks: &mut Vec<LocalTrack>) {
    tracks.sort_by(|a, b| {
        // Compare albums first
        match (&a.album, &b.album) {
            (Some(album_a), Some(album_b)) => {
                let cmp = album_a.cmp(album_b);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, None) => {}
        }
        // If albums are equal or both None, compare track numbers
        match (&a.number, &b.number) {
            (Some(num_a), Some(num_b)) => num_a.cmp(num_b),
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
}
