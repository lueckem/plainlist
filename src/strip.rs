use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use tracing::{error, info};

use crate::{
    library::{Library, QueryResult},
    track::{Track, scan_dir},
    utils::{PlaylistType, ensure_extension},
};

#[derive(Args)]
pub struct StripArgs {
    /// Path to the plainlist file
    input: PathBuf,

    /// Output path for the stripped plainlist
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Path to the music library directory
    #[arg(short, long)]
    library: PathBuf,
}

pub fn process_strip(args: StripArgs) -> anyhow::Result<()> {
    ensure_extension(&args.input, &PlaylistType::Plainlist)?;
    let output = if let Some(o) = args.output {
        ensure_extension(&o, &PlaylistType::Plainlist)?;
        o
    } else {
        // since input has the correct extension, this never panics
        let mut o = args.input.file_stem().unwrap().to_os_string();
        o.push("_stripped.plainlist");
        PathBuf::from(o)
    };

    let lib_path = args.library.canonicalize().context("invalid library")?;

    info!("Scanning library: '{}'", lib_path.display());
    let tracks = scan_dir(&lib_path);
    if tracks.is_empty() {
        error!("No tracks found in library");
        return Ok(());
    } else {
        info!("Library scan complete (found {} tracks)", tracks.len());
    }
    let library = Library::from_tracks(tracks);

    info!("Stripping missing tracks from '{}'", args.input.display());
    let content = std::fs::read_to_string(&args.input)?;
    let mut stripped_content = String::with_capacity(content.len());
    let mut num_stripped = 0;

    for (i, line) in content.lines().enumerate() {
        let l = line.trim();

        // comments and empty lines
        if l.starts_with('#') || l.is_empty() {
            stripped_content.push_str(line);
            stripped_content.push('\n');
            continue;
        }

        let track = l
            .parse::<Track>()
            .with_context(|| format!("invalid format in line {}: '{}'", i + 1, l))?;
        match library.query(&track) {
            QueryResult::NotFound => {
                num_stripped += 1;
                info!("line {}: '{}'", i + 1, l);
            }
            _ => {
                stripped_content.push_str(line);
                stripped_content.push('\n');
            }
        };
    }

    std::fs::write(&output, stripped_content)?;
    if num_stripped > 0 {
        info!("Deleted {} missing tracks", num_stripped);
    } else {
        info!("No missing tracks found; no tracks deleted");
    }
    info!("Created stripped plainlist '{}'", output.display());
    Ok(())
}
