use std::{io::Write, path::PathBuf};

use anyhow::Context;
use clap::Args;
use tracing::info;

use crate::{
    normalize::{NormalizedAlbum, NormalizedArtist},
    track::{LocalTrack, scan_dir},
    utils::{PlaylistType, ensure_extension, write_plainlist},
};

#[derive(Args)]
pub struct AppendArgs {
    /// Path to the input plainlist file
    input: PathBuf,

    /// Path to the music library directory
    #[arg(short, long)]
    library: PathBuf,

    /// Filter tracks by artist name
    #[arg(long)]
    artist: Option<String>,

    /// Filter tracks by album name
    #[arg(long)]
    album: Option<String>,
}

/// Append tracks from library to a plainlist
///
/// This function scans the music library and appends matching tracks (filtered by artist, album)
/// to an existing plainlist file.
pub fn process_append(args: AppendArgs) -> anyhow::Result<()> {
    ensure_extension(&args.input, &PlaylistType::Plainlist)?;

    // ask confirmation before adding the whole library
    if args.artist.is_none() && args.album.is_none() {
        if !ask_confirmation() {
            return Ok(());
        }
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

/// Scans the music library and filters tracks based on artist and album
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
///
/// Tracks without album or track number are sorted by title.
fn sort_tracks(tracks: &mut Vec<LocalTrack>) {
    tracks.sort_by(|a, b| {
        // Compare albums first
        match (&a.album, &b.album) {
            (Some(album_a), Some(album_b)) => {
                let cmp = album_a.to_lowercase().cmp(&album_b.to_lowercase());
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, None) => return a.track.title.cmp(&b.track.title),
        }
        // If albums are equal, compare disc and track numbers
        match (&a.disc, &b.disc) {
            (Some(num_a), Some(num_b)) => {
                let cmp = num_a.cmp(num_b);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            _ => (),
        }
        match (&a.number, &b.number) {
            (Some(num_a), Some(num_b)) => num_a.cmp(num_b),
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, None) => a.track.title.cmp(&b.track.title),
        }
    });
}

fn ask_confirmation() -> bool {
    loop {
        print!("No album/artist filter provided. Really append the whole library? [Y/n] ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        match input.to_lowercase().trim() {
            "" | "y" | "yes" => return true,
            "n" | "no" | "q" => return false,
            _ => (),
        };
    }
}
