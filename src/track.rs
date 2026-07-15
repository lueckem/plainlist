use std::{
    fmt::Display,
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::Context;
use lofty::{
    file::{FileType, TaggedFile, TaggedFileExt},
    probe::Probe,
    tag::{Accessor, ItemKey, Tag},
};
use regex::Regex;
use thiserror::Error;
use tracing::debug;

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

pub fn read_tracks_from_plainlist(path: impl AsRef<Path>) -> anyhow::Result<Vec<Track>> {
    let content = std::fs::read_to_string(path)?;

    let mut tracks = Vec::new();
    for (i, l) in content.lines().enumerate() {
        let l = l.trim();

        // skip comments and empty lines
        if l.starts_with('#') || l.is_empty() {
            continue;
        }
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

    pub fn read_from_file(path: &Path) -> anyhow::Result<Self> {
        let tagged_file = read_tagged_file(path)?;

        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
            .with_context(|| format!("no tags found for {}", path.display()))?;

        let title = tag
            .title()
            .map(|t| t.trim().to_owned())
            .filter(|t| !t.is_empty())
            .with_context(|| format!("missing or empty title for {}", path.display()))?;

        let artists = read_artists(tag)
            .with_context(|| format!("failed to get artists for {}", path.display()))?;

        Ok(Self::new(title, artists, path.to_path_buf()))
    }
}

fn read_tagged_file(path: &Path) -> anyhow::Result<TaggedFile> {
    let mut file =
        std::fs::File::open(path).with_context(|| format!("failed to open {}", path.display()))?;

    let tagged_file = if let Some(file_type) = FileType::from_path(path) {
        Probe::with_file_type(BufReader::new(file), file_type)
            .read()
            .with_context(|| format!("failed to get tags for {}", path.display()))?
    } else {
        debug!("Could not determine file type for {}", path.display());
        lofty::read_from(&mut file)
            .with_context(|| format!("failed to get tags for {}", path.display()))?
    };
    Ok(tagged_file)
}

/// Tries TrackArtists -> TrackArtist -> AlbumArtists -> AlbumArtist,
/// returning the first non-empty result.
fn read_artists(tag: &Tag) -> Option<Vec<String>> {
    read_multi(tag, ItemKey::TrackArtists)
        .or_else(|| read_single(tag, ItemKey::TrackArtist))
        .or_else(|| read_multi(tag, ItemKey::AlbumArtists))
        .or_else(|| read_single(tag, ItemKey::AlbumArtist))
}

fn read_multi(tag: &Tag, key: ItemKey) -> Option<Vec<String>> {
    let artists: Vec<String> = tag
        .get_strings(key)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    (!artists.is_empty()).then_some(artists)
}

fn read_single(tag: &Tag, key: ItemKey) -> Option<Vec<String>> {
    let artist = tag.get_string(key)?.trim();
    (!artist.is_empty()).then(|| split_artists(artist))
}

fn split_artists(artist: &str) -> Vec<String> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r";| / | feat\.? | ft\.? ").unwrap());

    RE.split(artist)
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .map(str::to_string)
        .collect()
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
