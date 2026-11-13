use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};

use crate::track::Track;

pub enum PlaylistType {
    /// Plainlist format (.plainlist extension)
    Plainlist,
    /// M3U playlist format (.m3u and .m3u8 extensions)
    M3U,
}

impl PlaylistType {
    /// Get the file extensions associated with this playlist type
    pub fn extensions(&self) -> Vec<&str> {
        match self {
            Self::Plainlist => vec!["plainlist"],
            Self::M3U => vec!["m3u", "m3u8"],
        }
    }

    /// Get the preferred file extension for this playlist type
    pub fn main_extension(&self) -> &str {
        self.extensions()[0]
    }
}

/// Collect input paths and target output paths with given types for conversion
///
/// This function handles the logic for mapping input files to output files based on
/// their types (e.g., Plainlist or M3U) and directory structures.
///
/// If `input` and `output` are files, the output path is just `output`.
/// If `input` is a file and `output` is `None`, the output path is the input stem with `out_ext`.
/// If `input` and `output` are directories, searches for all files with `in_ext` and puts the stems
/// in the `output` dir with the `out_ext` extension.
/// If `input` is a directory and `output` is `None`, searches for all files with `in_ext` and puts
/// the output paths next to them with same stem and `out_ext` extension.
///
/// Might return an empty vector if there are no files with `in_ext` in the directory.
pub fn collect_inputs_outputs(
    input: PathBuf,
    output: Option<PathBuf>,
    in_type: &PlaylistType,
    out_type: &PlaylistType,
) -> anyhow::Result<Vec<(PathBuf, PathBuf)>> {
    let out_ext = out_type.main_extension();

    // --- FILE ---
    if input.is_file() {
        ensure_extension(&input, in_type).context("wrong input file extension")?;

        let out_file = match output {
            None => input.with_extension(out_ext),
            Some(o) => {
                if o.is_dir() {
                    o.join(input.file_stem().unwrap()).with_extension(out_ext)
                } else if o.extension().is_none() {
                    o.with_extension(out_ext)
                } else {
                    ensure_extension(&o, out_type).context("wrong output file extension")?;
                    o
                }
            }
        };

        return Ok(vec![(input, out_file)]);

    // --- DIRECTORY ---
    } else if input.is_dir() {
        let in_files = collect_files_with_ptype(&input, in_type)?;
        if in_files.is_empty() {
            return Ok(Vec::new());
        }

        match output {
            None => {
                return Ok(in_files
                    .iter()
                    .map(|i| (i.to_owned(), i.with_extension(out_ext)))
                    .collect());
            }
            Some(o) => {
                if o.exists() && !o.is_dir() {
                    bail!(
                        "output '{}' exists and is not a directory (input was a directory, \
                     so output must be a directory too)",
                        o.display()
                    );
                }
                fs::create_dir_all(&o).with_context(|| {
                    format!("failed to create output directory '{}'", o.display())
                })?;

                return Ok(in_files
                    .iter()
                    .map(|i| {
                        (
                            i.to_owned(),
                            o.join(i.file_stem().unwrap()).with_extension(out_ext),
                        )
                    })
                    .collect());
            }
        }
    }

    bail!("input path '{}' does not exist", input.display())
}

/// Ensure a file has the correct extension for a playlist type
///
/// Validates that a file path has one of the valid extensions for the specified playlist type.
pub fn ensure_extension(path: &Path, ptype: &PlaylistType) -> anyhow::Result<()> {
    match path.extension() {
        Some(e) if ptype.extensions().iter().any(|v| *v == e) => Ok(()),
        _ => bail!(
            "file '{}' does not have a '.{}' extension",
            path.display(),
            ptype.main_extension(),
        ),
    }
}

/// Collect files with a specific playlist type from a directory
fn collect_files_with_ptype(dir: &Path, ptype: &PlaylistType) -> anyhow::Result<Vec<PathBuf>> {
    let valid_extensions = ptype.extensions();
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed reading directory '{}'", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|e| valid_extensions.iter().any(|v| *v == e))
        {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

/// Write track paths to an M3U playlist file
pub fn write_playlist_m3u<T, P>(track_paths: &[T], out_path: P) -> anyhow::Result<()>
where
    T: AsRef<Path>,
    P: AsRef<Path>,
{
    let mut content = String::new();
    for t in track_paths {
        content.push_str(&t.as_ref().to_string_lossy());
        content.push('\n');
    }
    std::fs::write(out_path, content)?;
    Ok(())
}

/// Write tracks to a plainlist file
///
/// The `append` argument controls whether to append to existing file or overwrite
pub fn write_plainlist<P>(tracks: &[Track], out_path: P, append: bool) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let mut content = String::new();
    for t in tracks {
        content.push_str(&t.to_string());
        content.push('\n');
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .open(out_path)?;
    file.write_all(content.as_ref())?;
    Ok(())
}
