use std::path::{Path, PathBuf};

/// Represents a binding between the jail and the machine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[allow(missing_docs)]
pub enum Bind {
    Dev { src: PathBuf, dest: PathBuf },
    ReadOnly { src: PathBuf, dest: PathBuf },
    ReadWrite { src: PathBuf, dest: PathBuf },
    TmpFs { path: PathBuf },
    Proc { path: PathBuf },
    Symlink { src: PathBuf, dest: PathBuf },
    Dir { path: PathBuf },
}

impl Bind {
    /// Creates a binding for a device.
    pub fn dev<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) -> Self {
        Self::Dev {
            src: src.as_ref().to_owned(),
            dest: dest.as_ref().to_owned(),
        }
    }

    /// Creates a read-only binding to a directory.
    pub fn read_only<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) -> Self {
        Self::ReadOnly {
            src: src.as_ref().to_owned(),
            dest: dest.as_ref().to_owned(),
        }
    }

    /// Creates a read-write binding to a directory.
    pub fn read_write<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) -> Self {
        Self::ReadWrite {
            src: src.as_ref().to_owned(),
            dest: dest.as_ref().to_owned(),
        }
    }

    /// Creates a binding for the temporal directory.
    pub fn tmp_fs<P: AsRef<Path>>(path: P) -> Self {
        Self::TmpFs {
            path: path.as_ref().to_owned(),
        }
    }

    /// Creates a binding for the proc filesystem.
    pub fn proc<P: AsRef<Path>>(path: P) -> Self {
        Self::Proc {
            path: path.as_ref().to_owned(),
        }
    }

    /// Creates a symlink between the source and destination.
    pub fn symlink<S: AsRef<Path>, D: AsRef<Path>>(src: S, dest: D) -> Self {
        Self::Symlink {
            src: src.as_ref().to_owned(),
            dest: dest.as_ref().to_owned(),
        }
    }

    /// Creates a new directory in the environemnt at the path.
    pub fn dir<P: AsRef<Path>>(path: P) -> Self {
        Self::Dir {
            path: path.as_ref().to_owned(),
        }
    }
}
