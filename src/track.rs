use std::{
    fmt::Display,
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::Context;
use lofty::{
    file::{FileType, TaggedFileExt},
    probe::Probe,
    tag::{Accessor, ItemKey, Tag},
};
use regex::Regex;
use thiserror::Error;
use tokio::fs::File;
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

    pub async fn read_from_file(path: &Path) -> anyhow::Result<Self> {
        let mut title: Option<String> = None;
        let mut artists: Vec<String> = Vec::new();

        let mut file = File::open(path).await?.into_std().await;
        let tagged_file = if let Some(file_type) = FileType::from_path(path) {
            Probe::with_file_type(BufReader::new(file), file_type)
                .read()
                .with_context(|| format!("failed to get tags for {}", path.display()))?
        } else {
            debug!("Could not determine file type for {}", path.display());
            lofty::read_from(&mut file)
                .with_context(|| format!("failed to get tags for {}", path.display()))?
        };

        // typically there is only 1 tag
        for tag in tagged_file.tags() {
            title = title.or_else(|| tag.title().map(|x| x.to_string()));
            append_artists_from_tag(&mut artists, tag);
        }

        let Some(title) = title else {
            anyhow::bail!("failed to get title for {}", path.display())
        };
        if title.is_empty() {
            anyhow::bail!("empty title for {}", path.display())
        }
        if artists.is_empty() {
            anyhow::bail!("failed to get artists for {}", path.display())
        };

        Ok(Self::new(title, artists, path.to_path_buf()))
    }
}

/// First, attempts to read the "TrackArtists".
/// If there are none, attempts to read "TrackArtist" and to separate into artists.
/// If still no artists are found, fall back to "AlbumArtists" and "AlbumArtist".
fn append_artists_from_tag(artists: &mut Vec<String>, tag: &Tag) {
    // FIXME: make sure that no empty artists are appended
    let len_before = artists.len();

    artists.extend(
        tag.get_strings(ItemKey::TrackArtists)
            .map(|s| s.trim().to_string()),
    );
    if artists.len() > len_before {
        return;
    }

    if let Some(artist) = tag.get_string(ItemKey::TrackArtist) {
        match separate_artists(artist) {
            None => artists.push(artist.trim().to_string()),
            Some(sep_artists) => artists.extend(sep_artists),
        }
        return;
    }

    // fall back to album artist
    artists.extend(
        tag.get_strings(ItemKey::AlbumArtists)
            .map(|s| s.trim().to_string()),
    );
    if artists.len() > len_before {
        return;
    }

    if let Some(artist) = tag.get_string(ItemKey::AlbumArtist) {
        match separate_artists(artist) {
            None => artists.push(artist.trim().to_string()),
            Some(sep_artists) => artists.extend(sep_artists),
        }
        return;
    }
}

/// "Artist1; Artist2" -> ["Artist1", "Artist2"]
/// "Artist1 feat. Artist2" -> ["Artist1", "Artist2"]
/// "Artist1 / Artist2" -> ["Artist1", "Artist2"]
///
/// If no separate artists were found, returns None.
fn separate_artists(artist: &str) -> Option<Vec<String>> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r";| / | feat\. | feat | ft\. | ft ").unwrap());

    let artists: Vec<&str> = RE
        .split(artist)
        .map(str::trim)
        .filter(|a| !a.is_empty())
        .collect();
    if artists.len() == 1 {
        return None; // there is only one artist
    }
    Some(artists.into_iter().map(str::to_string).collect())
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

    // TODO: test separate_artists
}
