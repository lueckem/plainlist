use std::path::PathBuf;

use anyhow::Context;
use clap::Args;

use crate::{
    track::read_tracks_from_plainlist,
    utils::{PlaylistType, ensure_extension},
};

#[derive(Args)]
pub struct InfoArgs {
    /// Path to the input plainlist file
    input: PathBuf,
}

/// Print information about a plainlist
pub fn process_info(args: InfoArgs) -> anyhow::Result<()> {
    ensure_extension(&args.input, &PlaylistType::Plainlist)?;
    let tracks = read_tracks_from_plainlist(&args.input)
        .with_context(|| format!("failed to parse plainlist '{}'", args.input.display()))?;

    // since it is a valid plainlist file, this never panics
    let plainlist_filename = args.input.file_name().unwrap();

    println!("--- '{}' ---", plainlist_filename.display());
    println!("Number of tracks: {}", tracks.len());

    Ok(())
}
