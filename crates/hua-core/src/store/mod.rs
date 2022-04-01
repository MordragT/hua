use snafu::prelude::*;
use std::path::PathBuf;

pub use backend::{Backend, LocalBackend};
pub use object::*;
pub use package::*;
pub use store::Store;

mod backend;
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
    #[snafu(display("Package could not be verified: {package}"))]
    PackageNotVerified { package: Package },
    #[snafu(display("Packge could not be found for {id}"))]
    PackageNotFoundById { id: ObjectId },
}

type StoreResult<T> = std::result::Result<T, StoreError>;
