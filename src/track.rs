use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub artists: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct ParsePlainError;

impl Track {
    pub fn new(title: String, artists: Vec<String>) -> Self {
        Self { title, artists }
    }

    /// Parse a string in the plainlist format.
    pub fn parse_from_plain(s: &str) -> Result<Self, ParsePlainError> {
        todo!();
    }
}

#[derive(Debug, Clone)]
pub struct LocalTrack {
    pub track: Track,
    pub path: PathBuf,
}

impl LocalTrack {
    pub fn new(title: String, artists: Vec<String>, path: PathBuf) -> Self {
        let track = Track::new(title, artists);
        Self { track, path }
    }
}
