use std::{
    io::{self, Error, ErrorKind},
    path::{Path, PathBuf},
};

/// A trait to allow replacing the file system lookup mechanisms.
///
/// As it stands, this is imperfect: it’s still using the types and some operations from
/// `std::path`, which constrain it to the target platform’s norms. This could be ameliorated by
/// the use of associated types for `Path` and `PathBuf`, and putting all remaining methods on this
/// trait (`is_absolute`, `parent`, `join`, *&c.*); but that would infect too many other APIs to be
/// desirable, so we live with it as it is—which is also acceptable, because the motivating example
/// use case is mostly using this as an optimisation over the real platform underneath.
pub trait Fs: std::fmt::Debug {
    /// Returns `true` if the path exists on disk and is pointing at a directory.
    fn is_dir(&self, path: &Path) -> bool;
    /// Returns `true` if the path exists on disk and is pointing at a regular file.
    fn is_file(&self, path: &Path) -> bool;
    /// Read the entire contents of a file into a bytes vector.
    fn read(&self, path: &Path) -> io::Result<Vec<u8>>;

    /// Canonicalize a file path
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        Ok(path.to_path_buf())
    }
}

/// Use [`std::fs`] to read any files from disk.
///
/// This is the default file system implementation.
#[derive(Debug)]
pub struct StdFs;

impl Fs for StdFs {
    #[inline]
    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    #[inline]
    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    #[inline]
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        std::fs::read(path)
    }

    #[inline]
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        std::fs::canonicalize(path)
    }
}

/// A file system implementation that acts like it’s completely empty.
///
/// This may be useful for security as it denies all access to the file system (so `@import` is
/// prevented from leaking anything); you’ll need to use [`from_string`][crate::from_string] for
/// this to make any sense (since [`from_path`][crate::from_path] would fail to find a file).
#[derive(Debug)]
pub struct NullFs;

impl Fs for NullFs {
    #[inline]
    fn is_file(&self, _path: &Path) -> bool {
        false
    }

    #[inline]
    fn is_dir(&self, _path: &Path) -> bool {
        false
    }

    #[inline]
    fn read(&self, _path: &Path) -> io::Result<Vec<u8>> {
        Err(Error::new(
            ErrorKind::NotFound,
            "NullFs, there is no file system",
        ))
    }
}
