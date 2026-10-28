use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};

use strsim::levenshtein;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

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
/// - Ambiguous: there are multiple best candidates with title "T" and non-empty artist intersection.
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

/// A candidate track (same normalized title as the query) together with
/// its artist overlap with the query. Only candidates with `overlap > 0` exist.
struct ScoredCandidate<'a> {
    track: &'a LocalTrack,
    artists: HashSet<NormalizedArtist>,
    /// Size of the intersection between the candidate's and the query's artists.
    overlap: usize,
}

impl ScoredCandidate<'_> {
    /// Larger overlap is better; for equal overlap, fewer artists is better.
    fn rank(&self) -> (usize, Reverse<usize>) {
        (self.overlap, Reverse(self.artists.len()))
    }
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
    /// 1. Look up the tracks whose normalized title equals that of `t` (the candidates).
    ///    If there are none, return NotFound.
    /// 2. Keep candidates that share at least one normalized artist with `t`, and
    ///    from those the best ranked ones: largest artist intersection, then smallest
    ///    artist set. If none are left, return NotFound.
    /// 3. If exactly one candidate is left, return Found (identical artist sets)
    ///    or FoundArtistMismatch.
    /// 4. Otherwise the match is ambiguous. If all remaining candidates have the same
    ///    artists, try to break the tie by title similarity (levenshtein distance).
    ///    If one candidate is strictly closest, return Found or FoundArtistMismatch.
    ///    Else, return Ambiguous.
    pub fn query(&self, t: &Track) -> QueryResult<'_> {
        let norm_title = NormalizedTitle::new(&t.title);
        let Some(candidates) = self.tracks.get(&norm_title) else {
            return QueryResult::NotFound;
        };

        let query_artists = norm_artist_set(&t.artists);
        let scored_candidates = score_candidates(candidates, &query_artists);
        let best = best_ranked(scored_candidates);

        match best.as_slice() {
            [] => QueryResult::NotFound,
            [only] => classify(only, &query_artists),
            _ => resolve_tie(&t.title, best, &query_artists),
        }
    }
}

/// Scores every candidate by artist overlap with the query and drops those without overlap.
fn score_candidates<'a>(
    candidates: &'a [LocalTrack],
    query_artists: &HashSet<NormalizedArtist>,
) -> Vec<ScoredCandidate<'a>> {
    candidates
        .iter()
        .filter_map(|lt| {
            let artists = norm_artist_set(&lt.track.artists);
            let overlap = artists.intersection(query_artists).count();
            (overlap > 0).then(|| ScoredCandidate {
                track: lt,
                artists,
                overlap,
            })
        })
        .collect()
}

/// Keeps only the candidates sharing the best rank. Preserves the original order.
fn best_ranked(mut scored: Vec<ScoredCandidate<'_>>) -> Vec<ScoredCandidate<'_>> {
    if let Some(best) = scored.iter().map(ScoredCandidate::rank).max() {
        scored.retain(|c| c.rank() == best);
    }
    scored
}

/// Turns a single chosen candidate into `Found` (identical artist sets)
/// or `FoundArtistMismatch`.
fn classify<'a>(
    candidate: &ScoredCandidate<'a>,
    query_artists: &HashSet<NormalizedArtist>,
) -> QueryResult<'a> {
    if &candidate.artists == query_artists {
        QueryResult::Found(candidate.track)
    } else {
        QueryResult::FoundArtistMismatch(candidate.track)
    }
}

/// Handles two or more equally ranked candidates.
///
/// The tie can only be broken by title similarity if all candidates have the
/// same artist set; otherwise (e.g. "{A, C}" vs "{A, D}") the result is ambiguous.
fn resolve_tie<'a>(
    query_title: &str,
    best: Vec<ScoredCandidate<'a>>,
    query_artists: &HashSet<NormalizedArtist>,
) -> QueryResult<'a> {
    if all_same_artists(&best) {
        if let Some(i) = unique_closest_title(query_title, &best) {
            return classify(&best[i], query_artists);
        }
    }
    QueryResult::Ambiguous(best.into_iter().map(|c| c.track).collect())
}

fn all_same_artists(candidates: &[ScoredCandidate<'_>]) -> bool {
    candidates.windows(2).all(|w| w[0].artists == w[1].artists)
}

/// Index of the candidate whose title is strictly closer to `query_title`
/// than every other candidate's, or `None` if there is a tie (or no candidates).
fn unique_closest_title(query_title: &str, candidates: &[ScoredCandidate<'_>]) -> Option<usize> {
    let query_title = normalize_title(query_title);
    let distances: Vec<usize> = candidates
        .iter()
        .map(|c| levenshtein(&query_title, &normalize_title(&c.track.track.title)))
        .collect();

    let min = *distances.iter().min()?;
    let mut at_min = distances.iter().enumerate().filter(|&(_, &d)| d == min);
    let (idx, _) = at_min.next()?;
    at_min.next().is_none().then_some(idx)
}

fn norm_artist_set(artists: &[String]) -> HashSet<NormalizedArtist> {
    artists.iter().map(|a| NormalizedArtist::new(a)).collect()
}

/// Normalize title by dealing with unicode and converting to lowercase.
///
/// It is different from NormalizedTitle as this keeps non-alpahanumeric characters
/// and does not remove suffixes like "- Remastered".
fn normalize_title(a: &str) -> String {
    a.trim()
        .nfd()
        .filter(|&c| !is_combining_mark(c))
        .flat_map(|c| c.to_lowercase())
        .collect()
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
            // suffix that is removed by normalization
            ("Hey Jude", vec!["The Beatles"]),
            ("Hey Jude - Remastered 2009", vec!["The Beatles"]),
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

    #[rstest]
    fn test_query_found_decided_by_title_similarity(library: Library) {
        // `Hey Jude - Remastered 2009` is more similar than `Hey Jude`
        let track: Track = "The Beatles -- Hey Jude - Remastered".parse().unwrap();

        let QueryResult::Found(lt) = library.query(&track) else {
            panic!("Expected QueryResult::Found")
        };
        assert_eq!(lt.track.title, "Hey Jude - Remastered 2009");
        assert_eq!(lt.track.artists, vec!["The Beatles"]);
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

    #[rstest]
    fn test_query_artist_mismatch_decided_by_title_similarity(library: Library) {
        // `Hey Jude - Remastered 2009` is more similar than `Hey Jude`,
        // but the artists do not match
        let track: Track = "The Beatles; John Lennon -- Hey Jude - Remastered"
            .parse()
            .unwrap();

        let QueryResult::FoundArtistMismatch(lt) = library.query(&track) else {
            panic!("Expected QueryResult::FoundArtistMismatch")
        };
        assert_eq!(lt.track.title, "Hey Jude - Remastered 2009");
        assert_eq!(lt.track.artists, vec!["The Beatles"]);
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

    #[rstest]
    fn test_query_ambiguous_exact_duplicates(library: Library) {
        // perfect match but the library holds two identical entries
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
