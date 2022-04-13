use snafu::prelude::*;
use std::path::PathBuf;

pub use backend::{Backend, LocalBackend};
pub use id::*;
pub use object::*;
pub use package::*;
pub use store::Store;

mod backend;
mod id;
mod object;
mod package;
mod store;

#[derive(Debug, Snafu)]
pub enum StoreError {
    #[snafu(display("Could not create rustbreak database: {source}"))]
    RustbreakCreateError { source: rustbreak::RustbreakError },
    #[snafu(display("Could not load rustbreak database: {source}"))]
    RustbreakLoadError { source: rustbreak::RustbreakError },
    #[snafu(display("Could not load data: {source}"))]
    RustbreakLoadDataError { source: rustbreak::RustbreakError },
    #[snafu(display("Could not save data: {source}"))]
    RustbreakSaveDataError { source: rustbreak::RustbreakError },
    #[snafu(display("Store does not exists at {path:#?}"))]
    NotExisting { path: PathBuf },
    #[snafu(display("IoError: {source}"))]
    IoError { source: std::io::Error },
    #[snafu(display("Could not link store objects from {original:#?} to {link:#?}: {source}"))]
    LinkObjectsError {
        kind: ObjectKind,
        original: PathBuf,
        link: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not copy object from {src:#?} to {dest:#?}: {source}"))]
    CopyObjectError {
        kind: ObjectKind,
        src: PathBuf,
        dest: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("Could not create tree at {path:#?}: {source}"))]
    CreateTreeError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[snafu(display("FsExtraError: {source}"))]
    FsExtraError { source: fs_extra::error::Error },
    #[snafu(display("Waldir Error: {source}"))]
    WalkDirError { source: walkdir::Error },
    #[snafu(display("Package could not be verified: {package}"))]
    PackageNotVerified { package: Package },
    #[snafu(display("Packge could not be found for {id}"))]
    PackageNotFoundById { id: PackageId },
    #[snafu(display("Object vould not be found for {id}"))]
    ObjectNotFoundById { id: ObjectId },
    #[snafu(display("A path or file name might have been invalid Utf-8"))]
    InvalidUtf8,
    #[snafu(display("StripPrefixError: {source}"))]
    StripPrefixError { source: std::path::StripPrefixError },
    #[snafu(display(
        "The following path cannot be linked, it must be in bin, lib, cfg or share: {path:#?}"
    ))]
    UnsupportedFilePath { path: PathBuf },
}

type StoreResult<T> = std::result::Result<T, StoreError>;
