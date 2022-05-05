use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    os::unix,
    path::{Path, PathBuf},
};

use crate::{GID, UID};

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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComponentPathBuf {
    pub binary: PathBuf,
    pub config: PathBuf,
    pub library: PathBuf,
    pub share: PathBuf,
}

impl ComponentPathBuf {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        Self {
            binary: path.join("bin"),
            config: path.join("cfg"),
            library: path.join("lib"),
            share: path.join("share"),
        }
    }

    pub fn global() -> Self {
        Self {
            binary: "/usr/bin".into(),
            config: "/etc".into(),
            library: "/usr/lib".into(),
            share: "/usr/share".into(),
        }
    }

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
