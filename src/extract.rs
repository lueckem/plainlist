use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use tracing::{debug, info};

use crate::{ExtractArgs, ExtractSource, track::LocalTrack};

pub fn process_extract(args: ExtractArgs) -> anyhow::Result<()> {
    let in_path = Path::new(&args.input);
    let out_path = args
        .output
        .map(|s| PathBuf::from(s).with_extension("plainlist"))
        .unwrap_or_else(|| in_path.with_extension("plainlist"));

    match args.from {
        ExtractSource::M3U => extract_m3u(in_path, &out_path),
        _ => anyhow::bail!("Not implemented yet!"),
    }
}

fn extract_m3u(in_path: &Path, out_path: &Path) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(in_path)?;
    let mut tracks = Vec::new();
    for (i, l) in content.lines().enumerate() {
        let l = l.trim();
        if l.is_empty() {
            continue;
        }
        // TODO: deal with M3U comments

        // TODO: instead of returning, log the error an continue
        let track_path = PathBuf::from_str(l)?;
        let lt = LocalTrack::read_from_file(track_path)?;
        debug!("Read track at {}", lt.path.display());
        tracks.push(lt);
    }

    let mut content = String::new();
    for lt in tracks {
        content.push_str(&lt.track.to_string());
        content.push('\n');
    }
    info!("Saving plainlist at {}", out_path.display());
    std::fs::write(out_path, content)?;

    Ok(())
}
