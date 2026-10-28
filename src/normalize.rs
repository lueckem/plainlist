//! String normalization utilities for music metadata

use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

/// Normalized song title
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedTitle(String);

impl NormalizedTitle {
    pub fn new(s: &str) -> Self {
        let normalized = normalize_str(s);
        Self(strip_title_suffixes(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Normalized artist name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedArtist(String);

impl NormalizedArtist {
    pub fn new(s: &str) -> Self {
        Self(normalize_str(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Normalized album name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedAlbum(String);

impl NormalizedAlbum {
    pub fn new(s: &str) -> Self {
        Self(normalize_str(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Normalize the string by removing non-alphanumeric characters, transforming to lower case,
/// and decomposing unicode (e.g., é to e).
///
/// For example "Rollin' (Air Raid Vehicle)" -> "rollinairraidvehicle".
fn normalize_str(s: &str) -> String {
    s.trim()
        .nfd()
        .filter(|&c| !is_combining_mark(c))
        .flat_map(|c| c.to_lowercase())
        .filter(|c| c.is_alphanumeric())
        .collect()
}

/// Strip common suffixes from title, for example
/// "sehnsuchtremastered2023" -> "sehnsucht".
fn strip_title_suffixes(title: String) -> String {
    // TODO: strip more common terms
    // TODO: it works with "Remastered 2001" but not "2001 Remaster"
    static RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(remastered|singleversion|radioversion|originalmix|radiomix)\d*$").unwrap()
    });
    let new_title = RE.replace(&title, "");
    if new_title.is_empty() {
        return title;
    }
    match new_title {
        std::borrow::Cow::Borrowed(_) => title,
        _ => new_title.into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("Metallica", "metallica")]
    #[case("ABBA", "abba")]
    #[case("DAYLIGHT", "daylight")]
    fn normalize_lowercase(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, normalize_str(input))
    }

    #[rstest]
    #[case("Avenged Sevenfold", "avengedsevenfold")]
    #[case("  System Of A Down ", "systemofadown")]
    #[case("Dirty Deeds Done Dirt Cheap", "dirtydeedsdonedirtcheap")]
    fn normalize_trim_whitespace(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, normalize_str(input))
    }

    #[rstest]
    #[case("It's My Life", "itsmylife")]
    #[case("It`s My Life", "itsmylife")]
    #[case("It’s My Life", "itsmylife")]
    #[case("(Don't Fear) The Reaper", "dontfearthereaper")]
    #[case("Rollin' (Air Raid Vehicle)", "rollinairraidvehicle")]
    #[case("Guitar Gangsters & Cadillac Blood", "guitargangsterscadillacblood")]
    #[case("Numb / Encore", "numbencore")]
    #[case("AC/DC", "acdc")]
    #[case("T.N.T", "tnt")]
    #[case("Chop Suey!", "chopsuey")]
    #[case("bbno$", "bbno")]
    #[case("Jay-Z", "jayz")]
    #[case("Jay\u{2011}Z", "jayz")]
    #[case("Jay\u{2013}Z", "jayz")]
    #[case("Jay\u{2014}Z", "jayz")]
    fn normalize_remove_non_alphanumeric(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, normalize_str(input))
    }

    #[rstest]
    #[case("Bamboléo", "bamboleo")]
    #[case("Beyoncé", "beyonce")]
    #[case("Mötley Crüe", "motleycrue")]
    #[case("Antonín Dvořák", "antonindvorak")]
    #[case("Beyonc\u{00E9}", "beyonce")]
    #[case("Beyonc\u{0065}\u{0301}", "beyonce")]
    fn normalize_unicode_decompose(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, normalize_str(input))
    }

    #[rstest]
    #[case("In The Air Tonight - Remastered 2015", "intheairtonight")]
    #[case("In The Air Tonight - Remastered", "intheairtonight")]
    #[case("Sehnsucht (Remastered 2023)", "sehnsucht")]
    #[case("Black or White - Single Version", "blackorwhite")]
    #[case("Dance with Somebody - Radio Version", "dancewithsomebody")]
    #[case("Remastered", "remastered")] // song "Remastered" should not be reduced to empty
    fn title_strip_suffixes(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, NormalizedTitle::new(input).as_str())
    }
}
