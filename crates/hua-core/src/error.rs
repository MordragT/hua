use std::{ffi::OsString, io};

use crate::package::Package;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    GenerationNotExistent(usize),
    PackageAlreadyPresent(Package),
    PackageNotFound(u64),
    TerminatingPath,
    UsernameNotFound,
    OsStringConversion,
    Io(io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenerationNotExistent(id) => f.write_str(&format!(
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
