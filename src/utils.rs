use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};

/// Collect input paths and target output paths with given extensions for conversion.
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
    in_ext: &str,
    out_ext: &str,
) -> anyhow::Result<Vec<(PathBuf, PathBuf)>> {
    // --- FILE ---
    if input.is_file() {
        ensure_extension(&input, in_ext).context("wrong input file extension")?;

        let out_file = match output {
            None => input.with_extension(out_ext),
            Some(o) => {
                if o.extension().is_none() {
                    o.with_extension(out_ext)
                } else {
                    ensure_extension(&o, out_ext).context("wrong output file extension")?;
                    o
                }
            }
        };

        return Ok(vec![(input, out_file)]);

    // --- DIRECTORY ---
    } else if input.is_dir() {
        let in_files = collect_files_with_extension(&input, in_ext)?;
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

fn ensure_extension(path: &Path, ext: &str) -> anyhow::Result<()> {
    match path.extension() {
        Some(e) if e == ext => Ok(()),
        _ => bail!(
            "file '{}' does not have a '.{}' extension",
            path.display(),
            ext
        ),
    }
}

/// Returns all files inside `dir` that have the given extension (not recursive).
fn collect_files_with_extension(dir: &Path, ext: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)
        .with_context(|| format!("failed reading directory '{}'", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|e| e == ext) {
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
