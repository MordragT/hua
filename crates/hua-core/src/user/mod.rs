use std::path::PathBuf;

use snafu::prelude::*;

mod manager;
mod user;

pub use manager::UserManager;
pub use user::User;

/// An [Error](std::error::Error) inside the [user](crate::user) module.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum UserError {
    #[snafu(display("The username could not be found"))]
    UsernameNotFound,
    #[snafu(display("IO Error: {source}"))]
    IoError { source: std::io::Error },
    #[snafu(display("Generation Error: {source}"))]
    GenerationError {
        source: crate::generation::GenerationError,
    },
    #[snafu(whatever, display("{message}"))]
    Whatever { message: String },
    #[snafu(display("Rusbreak {message}: {source}"))]
    RustbreakError {
        message: String,
        source: rustbreak::RustbreakError,
    },
    #[snafu(display("Path {path:#?} is not existing"))]
    PathNotExisting { path: PathBuf },
}

type UserResult<T> = Result<T, UserError>;
