use std::{
    fs,
    path::{Path, PathBuf, absolute},
    str::FromStr,
    sync::LazyLock,
};

use anyhow::Context;
use clap::{Args, ValueEnum};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::{
    track::{LocalTrack, Track, split_artists},
    utils::{PlaylistType, collect_inputs_outputs, write_plainlist},
};

/// Source format for extraction
#[derive(Debug, Clone, ValueEnum)]
enum ExtractSource {
    /// M3U playlist format
    M3U,
    /// Spotify export JSON format
    Spotify,
    /// JSON format for sharing playlists
    JSPF,
}

impl ExtractSource {
    /// Guess the extraction source from file extension
    fn guess_from_path(p: &Path) -> Option<Self> {
        let ext = p.extension()?.to_ascii_lowercase().into_string().ok()?;

        match ext.as_str() {
            "m3u" | "m3u8" => Some(Self::M3U),
            "jspf" => Some(Self::JSPF),
            _ => None,
        }
    }
}

#[derive(Args)]
pub struct ExtractArgs {
    /// Path to the input file
    input: PathBuf,

    /// Source format of the input file
    #[arg(value_enum, long)]
    from: Option<ExtractSource>,

    /// Output path for the generated plainlist
    #[arg(short)]
    output: Option<PathBuf>,
}

// TODO: document which formats work with directories
// and which only with single files

/// Extract plainlists from other formats
///
/// This function processes input files in various formats and converts them
/// to plainlist format. It determines the source type automatically when not specified.
pub fn process_extract(args: ExtractArgs) -> anyhow::Result<()> {
    let Some(extr_source) = args
        .from
        .clone()
        .or_else(|| ExtractSource::guess_from_path(&args.input))
    else {
        anyhow::bail!("could not determine input type");
    };

    // TODO: determine extr_source for each input output pair, not globally for all?

    match extr_source {
        ExtractSource::M3U => process_extract_m3u(args),
        ExtractSource::Spotify => process_extract_spotify(args),
        ExtractSource::JSPF => process_extract_jspf(args),
    }
}

// --------- M3U ------------

fn process_extract_m3u(args: ExtractArgs) -> anyhow::Result<()> {
    let in_path = absolute(args.input)?;
    let out_path = args.output.map(|p| absolute(p)).transpose()?;
    let in_out = collect_inputs_outputs(
        in_path,
        out_path,
        &PlaylistType::M3U,
        &PlaylistType::Plainlist,
    )?;
    if in_out.is_empty() {
        info!("No m3u or m3u8 playlists found.");
        return Ok(());
    }

    let mut k = 1;
    let num_playlists = in_out.len();

    for (i, o) in in_out {
        info!(
            "Extracting playlist [{}/{}]: {}",
            k,
            num_playlists,
            i.file_name().unwrap().display()
        );
        k += 1;

        extract_m3u(&i, &o)?;

        if k < num_playlists + 1 {
            println!("");
        }
    }

    Ok(())
}

/// Extract tracks from an M3U playlist file and write associated plainlist
fn extract_m3u(in_path: &Path, out_path: &Path) -> anyhow::Result<()> {
    // in_path is a file, so this never fails
    let in_dir = in_path.parent().unwrap();

    let content = std::fs::read_to_string(in_path)?;
    let mut num_invalid = 0;
    let mut tracks = Vec::new();
    for (i, l) in content.lines().enumerate() {
        let l = l.trim();

        // skip comments and empty lines
        if l.starts_with('#') || l.is_empty() {
            continue;
        }

        let Ok(mut track_path) = PathBuf::from_str(l);
        if track_path.is_relative() {
            track_path = in_dir.join(track_path);
        }

        let lt = match LocalTrack::read_from_file(track_path) {
            Ok(lt) => lt,
            Err(err) => {
                warn!("Invalid track path in line {}: {:#}", i + 1, err);
                num_invalid += 1;
                continue;
            }
        };
        debug!("Read track at {}", lt.path.display());
        tracks.push(lt);
    }

    let mut content = String::new();
    for lt in tracks.iter() {
        content.push_str(&lt.track.to_string());
        content.push('\n');
    }
    std::fs::write(out_path, content)?;
    info!(
        "Extracted {}/{} tracks",
        tracks.len(),
        tracks.len() + num_invalid
    );
    info!("Created plainlist '{}'", out_path.display());

    Ok(())
}

// --------- Spotify ------------

/// Extract tracks from a Spotify export JSON file and writes associated plainlists
fn process_extract_spotify(args: ExtractArgs) -> anyhow::Result<()> {
    if !args.input.is_file() {
        anyhow::bail!("input has to be a '.json' file");
    }
    let out_path = match args.output {
        None => args.input.parent().unwrap().to_path_buf(),
        Some(p) => {
            if p.exists() && !p.is_dir() {
                anyhow::bail!(
                    "output '{}' already exists and is not a directory",
                    p.display()
                );
            }
            fs::create_dir_all(&p)
                .with_context(|| format!("failed to create output directory '{}'", p.display()))?;
            p
        }
    };

    info!("Parsing '{}'", &args.input.display());
    let contents = fs::read_to_string(&args.input)?;
    let playlists: SpotifyPlaylists =
        serde_json::from_str(&contents).context("failed to parse file as Spotify JSON export")?;
    let playlists = playlists.playlists;
    info!("Parsed {} playlists", playlists.len());

    for (i, playlist) in playlists.iter().enumerate() {
        println!("");
        info!(
            "Extracting playlist [{}/{}]: '{}'",
            i + 1,
            playlists.len(),
            playlist.name
        );

        let tracks: Vec<Track> = playlist
            .items
            .iter()
            .map(|item| item.track.clone().into())
            .collect();
        info!("Extracted {} tracks", tracks.len());

        let o = out_path.join(&playlist.name).with_extension("plainlist");
        write_plainlist(&tracks, &o, false)?;
        info!("Created plainlist '{}'", o.display());
    }
    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct SpotifyPlaylists {
    playlists: Vec<SpotifyPlaylist>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpotifyPlaylist {
    name: String,
    items: Vec<SpotifyItem>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SpotifyItem {
    track: SpotifyTrack,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SpotifyTrack {
    track_name: String,
    artist_name: String,
}

impl From<SpotifyTrack> for Track {
    fn from(value: SpotifyTrack) -> Self {
        // Spotify typically stores only the main artist in `artist_name`
        // and the other artist in the `track_name` as "(feat. ...)"
        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"^(?P<title>.+?)\s*\(feat\.\s*(?P<artist>[^)]+)\)\s*$").unwrap()
        });
        let caps = RE.captures(&value.track_name);

        let (title, artists) = match caps {
            None => (value.track_name, vec![value.artist_name]),
            Some(c) => {
                let t = c["title"].trim().to_string();
                let a = vec![value.artist_name, c["artist"].trim().to_string()];
                debug!("{} -- {} ==> {:?} -- {}", a[0], value.track_name, a, t);
                (t, a)
            }
        };

        Track::new(title, artists)
    }
}

// --------- JSPF ------------

// TODO: should this only accept an input file or also directory?
fn process_extract_jspf(args: ExtractArgs) -> anyhow::Result<()> {
    if !args.input.is_file() {
        anyhow::bail!("input has to be a '.jspf' file");
    }
    // TODO: handle paths and names correctly
    if args.output.is_none() {
        anyhow::bail!("provide an output file name");
    }
    let out_path = args.output.unwrap();

    let contents = fs::read_to_string(&args.input)?;

    info!("Parsing '{}'", &args.input.display());
    let jspf_file: JSPFPFile =
        serde_json::from_str(&contents).context("failed to parse JSPF file")?;

    let playlist = &jspf_file.playlist;

    let tracks: Vec<Track> = playlist
        .track
        .iter()
        .map(|item| item.clone().into())
        .collect();
    info!("Extracted {} tracks", tracks.len());

    write_plainlist(&tracks, &out_path, false)?;
    info!("Created plainlist '{}'", out_path.display());

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct JSPFPFile {
    playlist: JSPFPlaylist,
}

#[derive(Serialize, Deserialize, Debug)]
struct JSPFPlaylist {
    track: Vec<JSPFTrack>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct JSPFTrack {
    title: String,
    creator: String,
}

impl From<JSPFTrack> for Track {
    fn from(value: JSPFTrack) -> Self {
        Track::new(value.title, split_artists(&value.creator))
    }
}
