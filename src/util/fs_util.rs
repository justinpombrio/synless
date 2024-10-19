use crate::util::{error, SynlessError};
use std::path::Path;

pub fn path_to_string(path: &Path) -> Result<String, SynlessError> {
    path.canonicalize()
        .map_err(|_| error!(FileSystem, "Invalid path: {}", path.display()))?
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
