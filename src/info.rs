use std::{
    cmp::Reverse,
    collections::{BTreeSet, HashMap},
    path::PathBuf,
};

use anyhow::Context;
use clap::Args;

use crate::{
    normalize::{NormalizedArtist, NormalizedTitle, norm_artist_btreeset},
    track::{Track, read_tracks_from_plainlist},
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

    println!("----- '{}' -----", plainlist_filename.display());
    println!("Number of songs: {}", tracks.len());
    println!("Number of artists: {}", artists.len());
    println!("");

    let dup_idxs = duplicate_tracks_idxs(&tracks);
    if !dup_idxs.is_empty() {
        println!("Number of duplicate songs: {}", dup_idxs.len());
        println!("Duplicate songs:");
        for group in dup_idxs {
            println!("{}  ({}x)", tracks[group[0]], group.len());
        }
        println!("");
    }

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

/// Find duplicate tracks.
///
/// Two tracks are considered duplicate if they have the same normalized title
/// and the same normalized artist sets.
///
/// Returns groups of indices into `tracks`. Each group has at least two
/// indices in ascending order, and groups are sorted by their first index.
fn duplicate_tracks_idxs(tracks: &[Track]) -> Vec<Vec<usize>> {
    let mut groups: HashMap<(NormalizedTitle, BTreeSet<NormalizedArtist>), Vec<usize>> =
        HashMap::new();

    for (i, t) in tracks.iter().enumerate() {
        let key = (
            NormalizedTitle::new(&t.title),
            norm_artist_btreeset(&t.artists),
        );
        groups.entry(key).or_default().push(i);
    }

    let mut duplicates: Vec<Vec<usize>> = groups.into_values().filter(|g| g.len() > 1).collect();
    duplicates.sort_unstable_by_key(|g| g[0]);
    duplicates
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(title: &str, artists: &[&str]) -> Track {
        Track::new(
            title.to_string(),
            artists.iter().map(|a| a.to_string()).collect(),
        )
    }

    #[test]
    fn test_duplicate_tracks_idxs() {
        let tracks = vec![
            track("Song A", &["Artist 1", "Artist 2"]),
            track("Song B", &["Artist 1", "Artist 2"]),
            track("Song A", &["Artist 3", "Artist 4"]),
            track("Song A", &["Artist 5", "Artist 6"]),
            track("Song A", &["Artist 1", "Artist 2"]),
            track("Song A", &["Artist 2", "Artist 1"]),
            track("Song A", &["Artist 1"]),
            track("Song B", &["Artist 1", "Artist 2"]),
            track("Song C", &["Artist 1", "Artist 2"]),
            track("Song A", &["Artist 5", "Artist 6"]),
        ];

        assert_eq!(
            duplicate_tracks_idxs(&tracks),
            vec![vec![0, 4, 5], vec![1, 7], vec![3, 9]]
        );
    }
}
