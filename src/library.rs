use std::collections::HashMap;

use crate::{normalize::NormalizedTitle, track::LocalTrack};

pub struct Library {
    tracks: HashMap<NormalizedTitle, Vec<LocalTrack>>,
}

impl Library {
    pub fn new() -> Self {
        Self {
            tracks: HashMap::new(),
        }
    }

    pub fn insert(&mut self, lt: LocalTrack) {
        let norm_title = NormalizedTitle::new(&lt.track.title);
        self.tracks.entry(norm_title).or_default().push(lt);
    }
}
