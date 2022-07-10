use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    os::unix,
    path::{Path, PathBuf},
};

use crate::{GID, UID};

/// Calculates the [RelativePathBuf] between two [Path].
///
/// # Example
///
/// ```no_run
/// use hua_core::extra::path;
/// use relative_path::RelativePathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let root = "/a/b/c/d";
/// let path = "c/d/e/f";
///
/// let relative = path::relative_path_between(root, path)?;
///
/// assert_eq!(relative, RelativePathBuf::from("e/f"));
/// # Ok(())
/// # }
pub fn relative_path_between<P: AsRef<Path>, Q: AsRef<Path>>(
    root: P,
    path: Q,
) -> io::Result<RelativePathBuf> {
    use io::{Error, ErrorKind};

    let absolute = path.as_ref().canonicalize()?;
    let relative = absolute
        .strip_prefix(root)
        .map_err(|err| Error::new(ErrorKind::Other, err))?;
    RelativePathBuf::from_path(relative).map_err(|err| Error::new(ErrorKind::Other, err))
}

/// A Path of different components provided by a [crate::store::Package] or [crate::generation::Generation].
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComponentPathBuf {
    /// The binary path
    pub binary: PathBuf,
    /// The config path
    pub config: PathBuf,
    /// The library path
    pub library: PathBuf,
    /// The share path
    pub share: PathBuf,
}

impl ComponentPathBuf {
    /// Creates a [ComponentPathBuf] from a path
    /// and creates the following component paths in there:
    /// - binary: `bin`
    /// - config: `cfg`
    /// - library: `lib`
    /// - share: `share`
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        Self {
            binary: path.join("bin"),
            config: path.join("cfg"),
            library: path.join("lib"),
            share: path.join("share"),
        }
    }

    /// Creates a [ComponentPathBuf] with global components:
    /// - binary: `/usr/bin`
    /// - config: `/etc/`
    /// - library: `/usr/lib`
    /// - share: `/usr/share`
    pub fn global() -> Self {
        Self {
            binary: "/usr/bin".into(),
            config: "/etc".into(),
            library: "/usr/lib".into(),
            share: "/usr/share".into(),
        }
    }

    /// Creates a new [ComponentPathBuf].
    pub fn new<T, U, V, W>(binary: T, config: U, library: V, share: W) -> Self
    where
        T: AsRef<Path>,
        U: AsRef<Path>,
        V: AsRef<Path>,
        W: AsRef<Path>,
    {
        Self {
            binary: binary.as_ref().to_owned(),
            config: config.as_ref().to_owned(),
            library: library.as_ref().to_owned(),
            share: share.as_ref().to_owned(),
        }
    }

    /// Creates the directories specified by the [ComponentPathBuf].
    /// If wanted those paths are then chowned by the [UID] and [GID].
    pub fn create_dirs(&self, chown: bool) -> io::Result<()> {
        if !self.binary.exists() {
            fs::create_dir(&self.binary)?;
            if chown {
                unix::fs::chown(&self.binary, UID, GID)?;
            }
        }
        if !self.config.exists() {
            fs::create_dir(&self.config)?;
            if chown {
                unix::fs::chown(&self.config, UID, GID)?;
            }
        }
        if !self.library.exists() {
            fs::create_dir(&self.library)?;
            if chown {
                unix::fs::chown(&self.library, UID, GID)?;
            }
        }
        if !self.share.exists() {
            fs::create_dir(&self.share)?;
            if chown {
                unix::fs::chown(&self.share, UID, GID)?;
            }
        }

        Ok(())
    }
}
