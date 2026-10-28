use std::collections::{HashMap, HashSet};

use crate::{
    normalize::{NormalizedArtist, NormalizedTitle},
    track::{LocalTrack, Track},
};

/// Result of querying the library for a track
///
/// Example: query track "A; B -- T"
/// - NotFound: there is no track with title "T" and artists containing "A" or "B".
/// - Found: unique and exact match with title "T" and artists "{A, B}".
/// - FoundArtistMismatch: uniquely best match with title "T" but artists do not match exactly,
///   e.g., "{A}" or "{A, C, D}". But the intersection of artists is non-empty.
///   Uniquely best means that it has a better match than all other tracks with title "T",
///   e.g., if there are two candidates with artists "{A, B, C}" and "{A, X, Y, Z}", then
///   "{A, B, C}" is the better match.
/// - Ambiguous: there a multiple best candidates with title "T" and non-empty artist intersection.
///   For example, two candidates with artists "{A, C}" and "{A, D}".
pub enum QueryResult<'a> {
    /// Exact match found
    Found(&'a LocalTrack),
    /// Match found but artist names don't exactly match
    FoundArtistMismatch(&'a LocalTrack),
    /// Multiple equally good matches found
    Ambiguous(Vec<&'a LocalTrack>),
    /// No match found
    NotFound,
}

/// A music library containing tracks indexed by normalized titles
pub struct Library {
    pub tracks: HashMap<NormalizedTitle, Vec<LocalTrack>>,
}

impl Library {
    pub fn new() -> Self {
        Self {
            tracks: HashMap::new(),
        }
    }

    pub fn from_tracks(tracks: Vec<LocalTrack>) -> Self {
        let mut l = Self {
            tracks: HashMap::with_capacity(tracks.len()),
        };

        for lt in tracks {
            l.insert(lt);
        }
        l
    }

    pub fn insert(&mut self, lt: LocalTrack) {
        let norm_title = NormalizedTitle::new(&lt.track.title);
        self.tracks.entry(norm_title).or_default().push(lt);
    }

    /// Query if track `t` is in the library
    ///
    /// Searches for a track in the library using normalized title and artist matching.
    /// Returns different match results based on how well the track matches.
    ///
    /// 1. Check if normalized title of `t` is in library. If not, return NotFound.
    /// 2. The vector of matches is called candidates.
    ///    Compare normalized artists of `t` against the candidates.
    ///    If there are no tracks in the candidates that have at least one matching artist, return NotFound.
    ///    If there are, find the matches with largest intersection and smallest artist set.
    /// 3. If the match is unique, return Found. Otherwise, return Ambiguous.
    pub fn query(&self, t: &Track) -> QueryResult<'_> {
        let norm_title = NormalizedTitle::new(&t.title);
        let Some(candidates) = self.tracks.get(&norm_title) else {
            return QueryResult::NotFound;
        };

        let query_artists = norm_artist_set(&t.artists);

        // (&track, intersect_size, artist_set_size)
        let mut scored_candidates: Vec<(&LocalTrack, usize, usize)> = candidates
            .iter()
            .filter_map(|lt| {
                let lt_artists = norm_artist_set(&lt.track.artists);
                let intersect_size = lt_artists.intersection(&query_artists).count();
                if intersect_size == 0 {
                    return None;
                }
                Some((lt, intersect_size, lt_artists.len()))
            })
            .collect();

        if scored_candidates.is_empty() {
            return QueryResult::NotFound;
        }

        // sort first by intersect_size, then by artist_set_size
        scored_candidates.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)));
        let (best_intersect, best_size) = (scored_candidates[0].1, scored_candidates[0].2);

        let matched_candidates: Vec<&LocalTrack> = scored_candidates
            .into_iter()
            .take_while(|(_, c, s)| *c == best_intersect && *s == best_size)
            .map(|(lt, _, _)| lt)
            .collect();
        if matched_candidates.len() == 1 {
            if (best_intersect == best_size) && (best_size == query_artists.len()) {
                return QueryResult::Found(matched_candidates[0]);
            } else {
                return QueryResult::FoundArtistMismatch(matched_candidates[0]);
            }
        }
        QueryResult::Ambiguous(matched_candidates)
    }
}

fn norm_artist_set(artists: &[String]) -> HashSet<NormalizedArtist> {
    artists.iter().map(|a| NormalizedArtist::new(a)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    // Helper function to create a LocalTrack for testing
    fn create_local_track(title: &str, artists: Vec<&str>) -> LocalTrack {
        let track = Track::new(
            title.to_string(),
            artists.into_iter().map(|s| s.to_string()).collect(),
        );
        LocalTrack::new(
            track.title,
            track.artists,
            None,
            None,
            None,
            std::path::PathBuf::from("test.mp3"),
        )
    }

    #[fixture]
    fn library() -> Library {
        let tracks = vec![
            // some example tracks
            ("Master of Puppets", vec!["Metallica"]),
            ("Numb / Encore", vec!["Linkin Park", "Jay-Z"]),
            ("Cheap Thrills", vec!["Sia"]),
            // same title, different artists
            ("Hurt", vec!["Nine Inch Nails"]),
            ("Hurt", vec!["Johnny Cash"]),
            // same title, shared first artist, different second artist
            ("Stay", vec!["Zedd", "Alessia Cara"]),
            ("Stay", vec!["Zedd", "Rihanna"]),
            // exact duplicates (e.g. same song on two albums)
            ("Hello", vec!["Adele"]),
            ("Hello", vec!["Adele"]),
            // candidates where the artist set is a subset/superset of another
            ("Sunflower", vec!["Post Malone", "Swae Lee"]),
            ("Sunflower", vec!["Post Malone", "Swae Lee", "DJ Snake"]),
            // same intersection with the query, but different artist set sizes
            ("Crowded Track", vec!["A", "B", "C"]),
            ("Crowded Track", vec!["A", "X", "Y", "Z"]),
        ];

        let mut library = Library::new();
        for (title, artists) in tracks {
            library.insert(create_local_track(title, artists));
        }
        library
    }

    // ---------- Test QueryResult::Found ----------

    #[rstest]
    fn test_query_found(library: Library) {
        let track: Track = "Metallica -- Master of Puppets".parse().unwrap();

        let QueryResult::Found(lt) = library.query(&track) else {
            panic!("Expected QueryResult::Found")
        };
        assert_eq!(lt.track.title, "Master of Puppets");
        assert_eq!(lt.track.artists, vec!["Metallica"]);
    }

    #[rstest]
    fn test_query_found_normalization(library: Library) {
        let track: Track = "METALLICA -- MASTER OF PUPPETS".parse().unwrap();

        let QueryResult::Found(lt) = library.query(&track) else {
            panic!("Expected QueryResult::Found")
        };
        assert_eq!(lt.track.title, "Master of Puppets");
        assert_eq!(lt.track.artists, vec!["Metallica"]);
    }

    #[rstest]
    fn test_query_found_artist_order_irrelevant(library: Library) {
        let track: Track = "Jay-Z; Linkin Park -- Numb / Encore".parse().unwrap();

        let QueryResult::Found(lt) = library.query(&track) else {
            panic!("Expected QueryResult::Found")
        };
        assert_eq!(lt.track.title, "Numb / Encore");
        assert_eq!(lt.track.artists, vec!["Linkin Park", "Jay-Z"]);
    }

    #[rstest]
    fn test_query_found_same_title_selects_by_artist(library: Library) {
        // must not match "Nine Inch Nails -- Hurt"
        let track: Track = "Johnny Cash -- Hurt".parse().unwrap();

        let QueryResult::Found(lt) = library.query(&track) else {
            panic!("Expected QueryResult::Found")
        };
        assert_eq!(lt.track.title, "Hurt");
        assert_eq!(lt.track.artists, vec!["Johnny Cash"]);
    }

    #[rstest]
    fn test_query_found_second_artist_disambiguates(library: Library) {
        // "Zedd" alone would be ambiguous, but "Zedd; Rihanna" has a larger
        // intersection with exactly one candidate.
        let track: Track = "Zedd; Rihanna -- Stay".parse().unwrap();

        let QueryResult::Found(lt) = library.query(&track) else {
            panic!("Expected QueryResult::Found")
        };
        assert_eq!(lt.track.artists, vec!["Zedd", "Rihanna"]);
    }

    #[rstest]
    fn test_query_found_prefers_exact_match_over_superset(library: Library) {
        // Both candidates have intersection 2; the smaller artist set wins.
        let track: Track = "Post Malone; Swae Lee -- Sunflower".parse().unwrap();

        let QueryResult::Found(lt) = library.query(&track) else {
            panic!("Expected QueryResult::Found")
        };
        assert_eq!(lt.track.artists, vec!["Post Malone", "Swae Lee"]);
    }

    #[rstest]
    fn test_query_found_prefers_exact_match_over_subset(library: Library) {
        // The 3-artist candidate has the larger intersection (3 vs 2).
        let track: Track = "Post Malone; Swae Lee; DJ Snake -- Sunflower"
            .parse()
            .unwrap();

        let QueryResult::Found(lt) = library.query(&track) else {
            panic!("Expected QueryResult::Found")
        };
        assert_eq!(
            lt.track.artists,
            vec!["Post Malone", "Swae Lee", "DJ Snake"]
        );
    }

    // ---------- Test QueryResult::FoundArtistMismatch ----------

    #[rstest]
    fn test_query_artist_mismatch_missing_artist(library: Library) {
        let track: Track = "Jay-Z -- Numb / Encore".parse().unwrap();

        let QueryResult::FoundArtistMismatch(lt) = library.query(&track) else {
            panic!("Expected QueryResult::FoundArtistMismatch")
        };
        assert_eq!(lt.track.title, "Numb / Encore");
        assert_eq!(lt.track.artists, vec!["Linkin Park", "Jay-Z"]);
    }

    #[rstest]
    fn test_query_artist_mismatch_additional_artist(library: Library) {
        let track: Track = "Sean Paul; Sia -- Cheap Thrills".parse().unwrap();

        let QueryResult::FoundArtistMismatch(lt) = library.query(&track) else {
            panic!("Expected QueryResult::FoundArtistMismatch")
        };
        assert_eq!(lt.track.title, "Cheap Thrills");
        assert_eq!(lt.track.artists, vec!["Sia"]);
    }

    #[rstest]
    fn test_query_artist_mismatch_partial_overlap(library: Library) {
        // Overlap on one artist, but each side has an artist the other lacks.
        let track: Track = "Linkin Park; Eminem -- Numb / Encore".parse().unwrap();

        let QueryResult::FoundArtistMismatch(lt) = library.query(&track) else {
            panic!("Expected QueryResult::FoundArtistMismatch")
        };
        assert_eq!(lt.track.artists, vec!["Linkin Park", "Jay-Z"]);
    }

    #[rstest]
    fn test_query_artist_mismatch_prefers_smaller_artist_set(library: Library) {
        // Both candidates have intersection 1; {A, B, C} beats {A, X, Y, Z}.
        let track: Track = "A -- Crowded Track".parse().unwrap();

        let QueryResult::FoundArtistMismatch(lt) = library.query(&track) else {
            panic!("Expected QueryResult::FoundArtistMismatch")
        };
        assert_eq!(lt.track.artists, vec!["A", "B", "C"]);
    }

    #[rstest]
    fn test_query_artist_mismatch_prefers_smaller_set_subset_query(library: Library) {
        // Query is a strict subset of both candidates.
        let track: Track = "Post Malone -- Sunflower".parse().unwrap();

        let QueryResult::FoundArtistMismatch(lt) = library.query(&track) else {
            panic!("Expected QueryResult::FoundArtistMismatch")
        };
        assert_eq!(lt.track.artists, vec!["Post Malone", "Swae Lee"]);
    }

    #[rstest]
    fn test_query_artist_mismatch_larger_intersection_beats_smaller(library: Library) {
        // {Post Malone, Swae Lee}: intersection 1, size 2
        // {Post Malone, Swae Lee, DJ Snake}: intersection 2, size 3  <- wins
        let track: Track = "Swae Lee; DJ Snake -- Sunflower".parse().unwrap();

        let QueryResult::FoundArtistMismatch(lt) = library.query(&track) else {
            panic!("Expected QueryResult::FoundArtistMismatch")
        };
        assert_eq!(
            lt.track.artists,
            vec!["Post Malone", "Swae Lee", "DJ Snake"]
        );
    }

    // ---------- Test QueryResult::Ambiguous ----------

    #[rstest]
    fn test_query_ambiguous_shared_artist(library: Library) {
        let track: Track = "Zedd -- Stay".parse().unwrap();

        let QueryResult::Ambiguous(candidates) = library.query(&track) else {
            panic!("Expected QueryResult::Ambiguous")
        };
        let mut artists: Vec<Vec<String>> = candidates
            .iter()
            .map(|lt| lt.track.artists.clone())
            .collect();
        artists.sort();
        assert_eq!(
            artists,
            vec![vec!["Zedd", "Alessia Cara"], vec!["Zedd", "Rihanna"]]
        );
    }

    #[rstest]
    fn test_query_ambiguous_query_matches_all_candidates_equally(library: Library) {
        // Each "Hurt" candidate matches exactly one query artist.
        let track: Track = "Nine Inch Nails; Johnny Cash -- Hurt".parse().unwrap();

        let QueryResult::Ambiguous(candidates) = library.query(&track) else {
            panic!("Expected QueryResult::Ambiguous")
        };
        assert_eq!(candidates.len(), 2);
        assert!(candidates.iter().all(|lt| lt.track.title == "Hurt"));
    }

    // TODO: Should this not actually give a QueryResult::Found?
    #[rstest]
    fn test_query_ambiguous_exact_duplicates(library: Library) {
        // Even a perfect artist match is ambiguous if the library holds two identical entries.
        let track: Track = "Adele -- Hello".parse().unwrap();

        let QueryResult::Ambiguous(candidates) = library.query(&track) else {
            panic!("Expected QueryResult::Ambiguous")
        };
        assert_eq!(candidates.len(), 2);
        for lt in candidates {
            assert_eq!(lt.track.title, "Hello");
            assert_eq!(lt.track.artists, vec!["Adele"]);
        }
    }

    // ---------- Test QueryResult::NotFound ----------

    #[rstest]
    fn test_query_not_found(library: Library) {
        let track: Track = "System Of A Down -- Chop Suey".parse().unwrap();
        assert!(matches!(library.query(&track), QueryResult::NotFound));
    }

    #[rstest]
    fn test_query_not_found_title_exists_but_not_artist(library: Library) {
        let track: Track = "Christina Aguilera -- Hurt".parse().unwrap();
        assert!(matches!(library.query(&track), QueryResult::NotFound));
    }

    #[rstest]
    fn test_query_not_found_artist_exists_but_not_title(library: Library) {
        let track: Track = "Metallica -- Enter Sandman".parse().unwrap();
        assert!(matches!(library.query(&track), QueryResult::NotFound));
    }
}
