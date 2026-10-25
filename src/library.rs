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
            ("Master of Puppets", vec!["Metallica"]),
            ("Numb / Encore", vec!["Linkin Park", "Jay-Z"]),
        ];

        let mut library = Library::new();
        for (title, artists) in tracks {
            library.insert(create_local_track(title, artists));
        }
        library
    }

    #[rstest]
    fn test_query_found(library: Library) {
        let track: Track = "Metallica -- Master Of Puppets".parse().unwrap();

        let QueryResult::Found(lt) = library.query(&track) else {
            panic!("Expected QueryResult::Found")
        };
        assert_eq!(lt.track.title, "Master of Puppets");
        assert_eq!(lt.track.artists, vec!["Metallica"]);
    }

    #[rstest]
    fn test_query_artist_mismatch(library: Library) {
        let track: Track = "Jay-Z -- Numb / Encore".parse().unwrap();

        let QueryResult::FoundArtistMismatch(lt) = library.query(&track) else {
            panic!("Expected QueryResult::FoundArtistMismatch")
        };
        assert_eq!(lt.track.title, "Numb / Encore");
        assert_eq!(lt.track.artists, vec!["Linkin Park", "Jay-Z"]);
    }

    // TODO: test ambiguous

    #[rstest]
    fn test_query_not_found(library: Library) {
        let track: Track = "System Of A Down -- Chop Suey".parse().unwrap();
        assert!(matches!(library.query(&track), QueryResult::NotFound));
    }

    // TODO: more tests
}
