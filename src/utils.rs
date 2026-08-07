use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};

pub enum PlaylistType {
    Plainlist,
    M3U,
}

impl PlaylistType {
    /// The file extensions that are associated to the type.
    pub fn extensions(&self) -> Vec<&str> {
        match self {
            Self::Plainlist => vec!["plainlist"],
            Self::M3U => vec!["m3u", "m3u8"],
        }
    }

    /// The preferred file extension of the type.
    pub fn main_extension(&self) -> &str {
        self.extensions()[0]
    }
}

/// Collect input paths and target output paths with given types for conversion.
///
/// If `input` and `output` are files, the output path is just `output`.
/// If `input` is a file and `output` is `None`, the output path is the input stem with`out_ext`.
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
                if o.extension().is_none() {
                    o.with_extension(out_ext)
                } else {
                    // TODO: Deal with the case that input is a file and output a dir
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
            // TODO: should this error on empty?
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

fn ensure_extension(path: &Path, ptype: &PlaylistType) -> anyhow::Result<()> {
    match path.extension() {
        Some(e) if ptype.extensions().iter().any(|v| *v == e) => Ok(()),
        _ => bail!(
            "file '{}' does not have a '.{}' extension",
            path.display(),
            ptype.main_extension(),
        ),
    }
}

/// Returns all files inside `dir` that have a valid extension for the PlaylistType (not recursive).
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

/// Writes the `track_paths` to disk at `out_path`.
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
