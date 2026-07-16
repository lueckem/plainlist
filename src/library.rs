use std::collections::HashMap;

use crate::{normalize::NormalizedTitle, track::LocalTrack};

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
}
