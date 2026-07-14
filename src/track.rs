use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::Context;
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

        let title = title.trim().to_string();
        if title.is_empty() {
            return Err(ParsePlainError::EmptyTitle);
        }

        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"; | / | feat\. ").unwrap());
        let artists: Vec<String> = RE
            .split(artists)
            .map(str::trim)
            .filter(|a| !a.is_empty())
            .map(str::to_string)
            .collect();
        if artists.is_empty() {
            return Err(ParsePlainError::EmptyArtist);
        }

        Ok(Track::new(title, artists))
    }
}

impl Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -- {}", self.artists.join("; "), self.title)
    }
}

pub async fn read_tracks_from_plainlist(path: impl AsRef<Path>) -> anyhow::Result<Vec<Track>> {
    let content = tokio::fs::read_to_string(path).await?;

    let mut tracks = Vec::new();
    for (i, l) in content.lines().enumerate() {
        let track = l
            .parse::<Track>()
            .with_context(|| format!("invalid format in line {}: '{}'", i + 1, l))?;
        tracks.push(track);
    }
    Ok(tracks)
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

    #[rstest]
    #[case("System Of A Down -- Chop Suey", "Chop Suey", vec!["System Of A Down"])]
    #[case("   System Of A Down    --     Chop Suey ", "Chop Suey", vec!["System Of A Down"])]
    #[case("AC/DC -- Highway to Hell", "Highway to Hell", vec!["AC/DC"])]
    #[case("Jay-Z; Linkin Park -- Numb / Encore", "Numb / Encore", vec!["Jay-Z", "Linkin Park"])]
    #[case("Jay-Z / Linkin Park -- Numb / Encore", "Numb / Encore", vec!["Jay-Z", "Linkin Park"])]
    #[case("Jay-Z feat. Linkin Park -- Numb / Encore", "Numb / Encore", vec!["Jay-Z", "Linkin Park"])]
    #[case("Earth, Wind & Fire -- September", "September", vec!["Earth, Wind & Fire"])]
    #[case("BABY GRAVY / Yung Gravy; bbno$ feat. Rich Brian -- C'est La Vie", "C'est La Vie", vec!["BABY GRAVY", "Yung Gravy", "bbno$", "Rich Brian"])]
    fn track_from_str(#[case] input: &str, #[case] title: &str, #[case] artists: Vec<&str>) {
        let track = Track::from_str(input).unwrap();
        assert_eq!(track.title, title);
        assert_eq!(track.artists, artists);
    }

    #[rstest]
    #[case("System Of A Down - Chop Suey")]
    #[case("System Of A Down--Chop Suey")]
    #[case("System Of A Down -- ")]
    #[case("System Of A Down --    ")]
    #[case(" -- Chop Suey")]
    #[case("   -- Chop Suey")]
    fn track_from_str_err(#[case] input: &str) {
        assert!(Track::from_str(input).is_err())
    }

    #[rstest]
    #[case("System Of A Down -- Chop Suey", "Chop Suey", vec!["System Of A Down"])]
    #[case("Jay-Z; Linkin Park -- Numb / Encore", "Numb / Encore", vec!["Jay-Z", "Linkin Park"])]
    #[case("BABY GRAVY; Yung Gravy; bbno$; Rich Brian -- C'est La Vie", "C'est La Vie", vec!["BABY GRAVY", "Yung Gravy", "bbno$", "Rich Brian"])]
    fn track_to_str(#[case] expected: &str, #[case] title: &str, #[case] artists: Vec<&str>) {
        // let artists: Vec<String> = artists.to_owned();
        let track = Track::new(
            title.to_owned(),
            artists.into_iter().map(str::to_string).collect(),
        );
        assert_eq!(track.to_string(), expected);
    }
}
