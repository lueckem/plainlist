use std::{
    cmp::Reverse,
    collections::{BTreeSet, HashMap},
    path::PathBuf,
};

use anyhow::Context;
use clap::Args;
use tabled::settings::{
    Alignment,
    object::{Columns, Segment},
};

use crate::{
    normalize::{NormalizedArtist, NormalizedTitle, norm_artist_btreeset},
    track::{Track, read_tracks_from_plainlist},
    utils::{PlaylistType, ensure_extension},
};

#[derive(Args)]
pub struct InfoArgs {
    /// Path to the plainlist file
    input: PathBuf,

    /// How many top artists to print
    #[arg(short, long, default_value_t = 10)]
    top_artists: usize,
}

/// Print information about a plainlist
pub fn process_info(args: InfoArgs) -> anyhow::Result<()> {
    ensure_extension(&args.input, &PlaylistType::Plainlist)?;
    let tracks = read_tracks_from_plainlist(&args.input)
        .with_context(|| format!("failed to parse plainlist '{}'", args.input.display()))?;

    // since it is a valid plainlist file, this never panics
    let plainlist_filename = args.input.file_name().unwrap();

    // (artist, count)
    let ac = artists_counts(&tracks);

    println!("----- '{}' -----", plainlist_filename.display());
    println!("");
    println!("Number of songs: {}", tracks.len());
    println!("Number of artists: {}", ac.len());
    println!("");

    // print duplicates if any
    print_duplicates(&tracks);

    // print top artists table
    if args.top_artists > 0 {
        print_top_artists(ac, tracks.len(), args.top_artists);
    }

    Ok(())
}

/// Extract all artists and their counts from tracks.
///
/// Returns a Vec with elements (artist, count),
/// sorted by count in descending order.
fn artists_counts(tracks: &[Track]) -> Vec<(String, usize)> {
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
    artists
}

/// Check if there are duplicates and print them
fn print_duplicates(tracks: &[Track]) {
    let dup_idxs = duplicate_tracks_idxs(&tracks);
    if !dup_idxs.is_empty() {
        println!("Number of duplicate songs: {}", dup_idxs.len());
        println!("Duplicate songs:");
        for group in dup_idxs {
            println!("{}  ({}x)", tracks[group[0]], group.len());
        }
        println!("");
    }
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

fn print_top_artists(ac: Vec<(String, usize)>, num_tracks: usize, num_top_artists: usize) {
    let num_tracks = num_tracks as f64;
    let mut builder = tabled::builder::Builder::new();

    // header
    builder.push_record(["", "Top artists", "# songs", "% songs"]);

    // push records
    let mut i: usize = 1;
    for (a, count) in ac.iter().take(num_top_artists) {
        let record: [String; 4] = [
            i.to_string(),
            a.clone(),
            count.to_string(),
            format!("{:.1}", *count as f64 / num_tracks * 100.0),
        ];
        builder.push_record(record);
        i += 1;
    }

    // create and style table
    let mut table = builder.build();
    table.modify(Segment::all(), Alignment::right());
    table.modify(Columns::one(1), Alignment::left());
    table.with(tabled::settings::Style::rounded());

    println!("{table}");
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
