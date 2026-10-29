//! String normalization utilities for music metadata

use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

/// Normalized song title
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NormalizedTitle(String);

impl NormalizedTitle {
    pub fn new(s: &str) -> Self {
        let normalized = normalize_str(strip_title_suffixes(s));
        if normalized.is_empty() {
            // Stripping suffixes left nothing; fall back to plain normalization.
            return Self(normalize_str(s));
        }
        Self(normalized)
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

/// Matches a trailing suffix that only describes a remaster or the original/radio/single/album
/// release of the same recording.
static SUFFIX_RE: LazyLock<Regex> = LazyLock::new(|| {
    // e.g. "'09", "1999", "2023"
    let year = r"(?:['’]\d{2}|(?:19|20)\d{2})";

    // "Remastered", "Remastered 2011", "2011 Remaster", "2009 Digital Remaster", etc.
    let remaster = format!(
        r"(?:{year}\s+)?(?:digital(?:ly)?\s+)?re-?master(?:ed|ing)?(?:\s+(?:version|edition|(?:in\s+)?{year}))*"
    );

    // "Single Version", "Radio Edit", "Album Version", "Original Mix", "Originalversion", etc.
    let variant = r"(?:(?:original|album|lp|single|radio|studio|club|extended)\s*)+(?:version|mix|edit|recording)|original|explicit(?:\s+version)?|bonus\s+track";

    let token = format!(r"(?:{remaster}|{variant})");
    // Several tokens in one segment: "Remastered 2011 / Single Version"
    let segment = format!(r"{token}(?:[\s,/&+-]+{token})*");

    let pattern =
        format!(r"(?i)(?:\s+[-–—]\s+{segment}|\s*[(\[]\s*{segment}\s*[)\]]|\s+{remaster})\s*$");
    Regex::new(&pattern).unwrap()
});

/// Strips trailing suffixes like "- Remastered 2011" or "(Single Version)".
/// Repeats until nothing is left to strip, so "Song (Remastered 2011) - Single Version" is stripped
/// to "Song".
fn strip_title_suffixes(title: &str) -> &str {
    let mut rest = title.trim();
    while let Some(m) = SUFFIX_RE.find(rest) {
        let head = rest[..m.start()].trim_end();
        if head.is_empty() {
            break; // never strip down to nothing
        }
        rest = head;
    }
    rest
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
    // remasters
    #[case("In The Air Tonight - Remastered 2015", "intheairtonight")]
    #[case("In The Air Tonight - Remastered", "intheairtonight")]
    #[case("In The Air Tonight - Remastered '15", "intheairtonight")]
    #[case("Sehnsucht (Remastered 2023)", "sehnsucht")]
    #[case("Sehnsucht (Remastered)", "sehnsucht")]
    #[case("Sehnsucht [2023 Remaster]", "sehnsucht")]
    #[case("Sehnsucht (Remastered in 2023)", "sehnsucht")]
    #[case("Another Brick in the Wall - 2011 Remaster", "anotherbrickinthewall")]
    #[case("Song – 2009 Digital Remaster", "song")]
    #[case("Song – Digitally Remastered", "song")]
    #[case("Song Remastered 2011", "song")]
    #[case("Apollo 11 Remastered", "apollo11")]
    // original / radio / single / album / etc.
    #[case("Black or White - Single Version", "blackorwhite")]
    #[case("Dance with Somebody - Radio Version", "dancewithsomebody")]
    #[case("Song (Radio Edit)", "song")]
    #[case("Song - Album Version", "song")]
    #[case("Song - Original Mix", "song")]
    #[case("Song (Explicit)", "song")]
    #[case("Song (Bonus Track)", "song")]
    #[case("Song (Extended Mix)", "song")]
    #[case("Song (Original Extended Mix)", "song")]
    #[case("Song (Club Mix)", "song")]
    // combinations
    #[case("Song - Single Version (Remastered 2011)", "song")]
    #[case("Song (Radio Edit / Remastered)", "song")]
    #[case("Song - Remastered 2011 - Single Version", "song")]
    // must NOT be reduced to empty
    #[case("Remastered", "remastered")]
    #[case("Remastered 2011", "remastered2011")]
    #[case("(Remastered)", "remastered")]
    #[case("Remastered (Remastered 2011)", "remastered")]
    // different recordings: must be kept
    #[case("Song - Live", "songlive")]
    #[case("Song (Live in Berlin)", "songliveinberlin")]
    #[case("Song (Techno Remix)", "songtechnoremix")]
    #[case("Song - Acoustic Version", "songacousticversion")]
    #[case("Song - Live at Wembley - Remastered", "songliveatwembley")]
    // must not trigger on similar names
    #[case("The Single Version", "thesingleversion")]
    #[case("Truly Original Mix", "trulyoriginalmix")]
    fn title_strip_suffixes(#[case] input: &str, #[case] expected: &str) {
        assert_eq!(expected, NormalizedTitle::new(input).as_str())
    }
}
