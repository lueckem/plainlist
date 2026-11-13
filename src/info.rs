use std::{cmp::Reverse, collections::HashMap, path::PathBuf};

use anyhow::Context;
use clap::Args;

use crate::{
    normalize::NormalizedArtist,
    track::read_tracks_from_plainlist,
    utils::{PlaylistType, ensure_extension},
};

#[derive(Args)]
pub struct InfoArgs {
    /// Path to the input plainlist file
    input: PathBuf,
}

/// Print information about a plainlist
pub fn process_info(args: InfoArgs) -> anyhow::Result<()> {
    ensure_extension(&args.input, &PlaylistType::Plainlist)?;
    let tracks = read_tracks_from_plainlist(&args.input)
        .with_context(|| format!("failed to parse plainlist '{}'", args.input.display()))?;

    // norm_artist -> (artist, count)
    let mut artists: HashMap<NormalizedArtist, (String, usize)> = HashMap::new();
    for t in tracks.iter() {
        for a in t.artists.iter() {
            let norm_a = NormalizedArtist::new(a);
            artists.entry(norm_a).or_insert((a.clone(), 0)).1 += 1;
        }
    }
    // (artist, count)
    let mut artists: Vec<(String, usize)> = artists.values().cloned().collect();
    artists.sort_unstable_by_key(|a| Reverse(a.1));

    // since it is a valid plainlist file, this never panics
    let plainlist_filename = args.input.file_name().unwrap();

    println!("--- '{}' ---", plainlist_filename.display());
    println!("Number of tracks: {}", tracks.len());
    println!("Number of artists: {}", artists.len());
    println!("");

    let padding = artists
        .iter()
        .take(10)
        .map(|(a, _)| a.chars().count())
        .max()
        .unwrap_or(0);

    let header = format!("{:<padding$} -  #songs", "Top artists", padding = padding);
    println!("{header}");
    println!("{}", "-".repeat(header.chars().count()));
    for (a, count) in artists.iter().take(10) {
        println!("{:<padding$} -  {}", a, count, padding = padding);
    }

    Ok(())
}
