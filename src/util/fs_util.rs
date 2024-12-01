use crate::util::{error, SynlessError};
use std::path::{Path, PathBuf};

pub fn path_to_string(path: &Path) -> Result<String, SynlessError> {
    let mut path_buf = PathBuf::from(path);
    let mut suffixes = Vec::new();
    while !path_buf.exists() {
        if let Some(suffix) = path_buf.file_name() {
            suffixes.push(suffix.to_owned());
            path_buf.pop();
        } else {
            return Err(error!(FileSystem, "Path is confusing: {}", path.display()));
        }
    }
    let mut canonical_path = path_buf
        .canonicalize()
        .map_err(|_| error!(FileSystem, "Invalid path: {}", path.display()))?;
    while let Some(suffix) = suffixes.pop() {
        canonical_path.push(suffix);
    }
    canonical_path
        .to_str()
        .map(|s| s.to_owned())
        .ok_or_else(|| {
            error!(
                FileSystem,
                "Path is not valid unicode: {}",
                path.to_string_lossy()
            )
        })
}

pub fn path_file_name(path: &str) -> Result<String, SynlessError> {
    let os_str = Path::new(path)
        .file_name()
        .ok_or_else(|| error!(FileSystem, "Path ends in `..`: {path}"))?;

    Ok(os_str
        .to_str()
        .ok_or_else(|| error!(FileSystem, "Path is not valid unicode: {path}"))?
        .to_owned())
}

pub fn join_path(path_1: &str, path_2: &str) -> Result<String, SynlessError> {
    path_to_string(&Path::new(path_1).join(path_2))
}
