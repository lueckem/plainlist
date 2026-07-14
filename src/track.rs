use std::{path::PathBuf, str::FromStr};

use regex::Regex;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub artists: Vec<String>,
}

impl Track {
    pub fn new(title: String, artists: Vec<String>) -> Self {
        Self { title, artists }
    }
}

#[derive(Error, Debug)]
pub enum ParsePlainError {
    #[error("missing artist and title separator ' -- '")]
    MissingSeparator,
    #[error("empty title")]
    EmptyTitle,
    #[error("empty artist")]
    EmptyArtist,
}

impl FromStr for Track {
    type Err = ParsePlainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (artists, title) = s
            .split_once(" -- ")
            .ok_or(ParsePlainError::MissingSeparator)?;
        if title.is_empty() {
            return Err(ParsePlainError::EmptyTitle);
        }

        let re = Regex::new(r"; | / | feat\. ").unwrap();

        let artists: Vec<String> = re
            .split(artists)
            .map(str::trim)
            .filter(|a| !a.is_empty())
            .map(str::to_string)
            .collect();
        if artists.is_empty() {
            return Err(ParsePlainError::EmptyArtist);
        }

        Ok(Track::new(title.to_string(), artists))
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // TODO: more tests
    #[rstest]
    #[case("System Of A Down -- Chop Suey", "Chop Suey", vec!["System Of A Down"])]
    #[case("AC/DC -- Highway to Hell", "Highway to Hell", vec!["AC/DC"])]
    #[case("Jay-Z; Linkin Park -- Numb / Encore", "Numb / Encore", vec!["Jay-Z", "Linkin Park"])]
    #[case("Jay-Z / Linkin Park -- Numb / Encore", "Numb / Encore", vec!["Jay-Z", "Linkin Park"])]
    #[case("Jay-Z feat. Linkin Park -- Numb / Encore", "Numb / Encore", vec!["Jay-Z", "Linkin Park"])]
    fn track_from_str(#[case] input: &str, #[case] title: &str, #[case] artists: Vec<&str>) {
        let track = Track::from_str(input).unwrap();
        assert_eq!(track.title, title);
        assert_eq!(track.artists, artists);
    }

    // TODO: tests track_from_str_err
}
