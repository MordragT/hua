use std::{ffi::OsString, io, path::PathBuf};

use daggy::NodeIndex;
use relative_path::RelativePathBuf;
use semver::{Version, VersionReq};

use crate::{Package, Requirement};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    RequirementNameCollision(String),
    NoRequiredMatches(Requirement),
    GenerationIsInUse,
    GenerationNotFound(usize),
    GenerationAlreadyPresent(usize),
    PackageAlreadyPresent(u64),
    PackageNotFoundByIndex(u64),
    PackageNotFound(Package),
    DependencyProvideCollision(Requirement),
    VersionMissmatch((Version, VersionReq)),
    HashNotFound(NodeIndex<usize>),
    IndexNotFound(u64),
    TerminatingPath,
    PathNotFound(PathBuf),
    WrongPathParent(RelativePathBuf),
    UsernameNotFound,
    OsStringConversion,
    Io(io::Error),
    DBNotRecovered(String),
    Pot(pot::Error),
    Rustbreak(rustbreak::RustbreakError),
    Reqwest(reqwest::Error),
    Url(url::ParseError),
    RecipeNoUnpackSource,
    Cycle(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cycle(s) => f.write_str(&s),
            Self::NoRequiredMatches(r) => f.write_str(&format!("No required matches found: {}", r)),
            Self::RequirementNameCollision(s) => f.write_str(&format!(
                "Requirements could not be resolved due to colliding name: {}",
                s
            )),
            Self::WrongPathParent(p) => f.write_str(&format!(
                "The following relative path had a wrong parent dir: {}",
                p
            )),
            Self::VersionMissmatch((ver, req)) => {
                f.write_str(&format!("Version missmatch: {}, required: {}", ver, req))
            }
            Self::DependencyProvideCollision(dep) => f.write_str(&format!(
                "The provided functionality of the following dependency collides {}",
                dep
            )),
            Self::RecipeNoUnpackSource => f.write_str("No source to unpack found"),
            Self::GenerationIsInUse => f.write_str("Cannot delete the current generation"),
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
            Self::PackageNotFound(p) => {
                f.write_str(&format!("The package could not be found: {p}"))
            }
            Self::PackageNotFoundByIndex(i) => {
                f.write_str(&format!("Package not found by index {i}"))
            }
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
            Self::Reqwest(e) => f.write_str(&e.to_string()),
            Self::Url(e) => f.write_str(&e.to_string()),
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

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<daggy::WouldCycle<Vec<&Requirement>>> for Error {
    fn from(e: daggy::WouldCycle<Vec<&Requirement>>) -> Self {
        Self::Cycle(e.to_string())
    }
}
