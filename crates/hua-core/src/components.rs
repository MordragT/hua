use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};
use std::{
    fmt, fs,
    hash::Hash,
    io,
    path::{Path, PathBuf},
};

#[derive(PartialEq, Eq, Debug, Hash, Serialize, Deserialize, Clone, PartialOrd, Ord)]
pub enum Binary {
    Shell(RelativePathBuf),
    Elf(RelativePathBuf),
}

impl Binary {
    /// Return the inner relative path
    pub fn relative_path(&self) -> &RelativePath {
        match self {
            Self::Shell(path) => &path,
            Self::Elf(path) => &path,
        }
    }
}

impl Into<RelativePathBuf> for Binary {
    fn into(self) -> RelativePathBuf {
        match self {
            Self::Shell(path) => path,
            Self::Elf(path) => path,
        }
    }
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shell(p) => f.write_str(&format!("Shell {p}")),
            Self::Elf(p) => f.write_str(&format!("Elf {p}")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum Component {
    Binary(Binary),
    Config(RelativePathBuf),
    Library(RelativePathBuf),
    Share(RelativePathBuf),
}

impl Component {
    /// Gets the underlying relative path
    pub fn relative_path(&self) -> &RelativePath {
        match &self {
            Self::Binary(b) => b.relative_path(),
            Self::Config(p) | Self::Library(p) | Self::Share(p) => &p,
        }
    }
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binary(b) => f.write_str(&format!("Binary {b}")),
            Self::Config(p) => f.write_str(&format!("Config {p}")),
            Self::Library(p) => f.write_str(&format!("Library {p}")),
            Self::Share(p) => f.write_str(&format!("Share {p}")),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComponentPaths {
    pub binary: PathBuf,
    pub config: PathBuf,
    pub library: PathBuf,
    pub share: PathBuf,
}

impl ComponentPaths {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        Self {
            binary: path.join("bin"),
            config: path.join("cfg"),
            library: path.join("lib"),
            share: path.join("share"),
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

    pub fn create_dirs(&self) -> io::Result<()> {
        if !self.binary.exists() {
            fs::create_dir(&self.binary)?;
        }
        if !self.config.exists() {
            fs::create_dir(&self.config)?;
        }
        if !self.library.exists() {
            fs::create_dir(&self.library)?;
        }
        if !self.share.exists() {
            fs::create_dir(&self.share)?;
        }
        Ok(())
    }
}
