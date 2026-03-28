use std::{
    env,
    path::{Component, Path, PathBuf},
    io,
};

/// Error type for path operations
#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Invalid UTF-8 in path")]
    InvalidUtf8,
}

/// Resolves a relative path to an absolute path using the current working directory
///
/// # Arguments
/// * `path` - The relative path to resolve
///
/// # Returns
/// * `Result<String, PathError>` - The resolved absolute path as a string
pub fn resolve_path(path: impl AsRef<str>) -> Result<String, PathError> {
    let current_dir = env::current_dir()?;
    let mut absolute_path = current_dir;
    absolute_path.push(path.as_ref());
    
    absolute_path
        .into_os_string()
        .into_string()
        .map_err(|_| PathError::InvalidUtf8)
}

/// Joins multiple path parts into a single canonical path
///
/// # Arguments
/// * `parts` - Slice of path parts to join
///
/// # Returns
/// * `Result<String, PathError>` - The joined and canonicalized path as a string
pub fn path_join(parts: &[&str]) -> Result<String, PathError> {
    let mut path_buf = PathBuf::new();
    for part in parts {
        path_buf.push(part);
    }
    
    Ok(path_buf.canonicalize()?.to_string_lossy().into_owned())
}

/// Normalizes a path by resolving parent directory references and maintaining trailing slashes
///
/// # Arguments
/// * `path` - The path to normalize
///
/// # Returns
/// * `PathBuf` - The normalized path
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let ends_with_slash = path
        .as_ref()
        .to_str()
        .map_or(false, |s| s.ends_with('/'));
    
    let mut normalized = PathBuf::new();
    for component in path.as_ref().components() {
        match component {
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component);
                }
            }
            _ => normalized.push(component),
        }
    }
    
    if ends_with_slash {
        normalized.push("");
    }
    
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path("a/b/../c"),
            PathBuf::from("a/c")
        );
        assert_eq!(
            normalize_path("a/b/../../c"),
            PathBuf::from("c")
        );
        assert_eq!(
            normalize_path("a/b/c/"),
            PathBuf::from("a/b/c/")
        );
    }
}
