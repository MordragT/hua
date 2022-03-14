use std::{ffi::OsString, io, path::PathBuf};

use daggy::NodeIndex;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    GenerationNotFound(usize),
    GenerationAlreadyPresent(usize),
    PackageAlreadyPresent(u64),
    PackageNotFound(u64),
    HashNotFound(NodeIndex<usize>),
    IndexNotFound(u64),
    TerminatingPath,
    PathNotFound(PathBuf),
    UsernameNotFound,
    OsStringConversion,
    Io(io::Error),
    DBNotRecovered(String),
    Pot(pot::Error),
    Rustbreak(rustbreak::RustbreakError),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathNotFound(p) => {
                f.write_str(&format!("The given path was not found: {:#?}", p))
            }
            Self::GenerationAlreadyPresent(id) => f.write_str(&format!(
                "The generation with the following id is already present: {}",
                id
            )),
            Self::GenerationNotFound(id) => f.write_str(&format!(
                "The generation with the following id is not existent: {}",
                id
            )),
            Self::TerminatingPath => f.write_str("The path given terminates at ./. or /"),
            Self::PackageNotFound(h) => f.write_str(&format!(
                "The package with the following hash could not be found: {}",
                h
            )),
            Self::PackageAlreadyPresent(p) => {
                f.write_str(&format!("The package was already present: {}", p))
            }
            Self::OsStringConversion => {
                f.write_str("An os string could not be converted due to invalid unicode data")
            }
            Self::UsernameNotFound => f.write_str("The current username could not be retrieved"),
            Self::Io(e) => f.write_str(&e.to_string()),
            Self::DBNotRecovered(db) => f.write_str(&format!(
                "The given database: {}, was not recovered but created newly",
                db
            )),
            Self::Pot(e) => f.write_str(&e.to_string()),
            Self::Rustbreak(e) => f.write_str(&e.to_string()),
            Self::HashNotFound(idx) => f.write_str(&format!(
                "A hash for the follwing index was not found: {}",
                idx.index()
            )),
            Self::IndexNotFound(hash) => f.write_str(&format!(
                "An index for the follwing hash was not found: {}",
                hash
            )),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<OsString> for Error {
    fn from(_: OsString) -> Self {
        Self::OsStringConversion
    }
}

impl From<pot::Error> for Error {
    fn from(e: pot::Error) -> Self {
        Self::Pot(e)
    }
}

impl From<rustbreak::RustbreakError> for Error {
    fn from(e: rustbreak::RustbreakError) -> Self {
        Self::Rustbreak(e)
    }
}
