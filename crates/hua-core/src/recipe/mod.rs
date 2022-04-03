use snafu::prelude::*;

pub use recipe::Recipe;

use crate::GenerationError;

mod recipe;

pub const LINUX: u8 = 0x01;
pub const X86_64: u8 = 0x01;

#[derive(Debug, Snafu)]
pub enum RecipeError {
    #[snafu(display("CacheError: {source}"))]
    CacheError { source: cached_path::Error },
    #[snafu(display("IoError: {source}"))]
    IoError { source: std::io::Error },
    #[snafu(display("FsExtraError: {source}"))]
    FsExtraError { source: fs_extra::error::Error },
    #[snafu(display("GenerationError: {source}"))]
    GenerationError { source: GenerationError },
    #[snafu(display("Fetch the source files first"))]
    MissingSourceFiles,
    #[snafu(display("Prepare requirements first"))]
    MissingJail,
    #[snafu(display("The recipe is not compatible with your architecture"))]
    IncompatibleArchitecture,
    #[snafu(display("The recipe is not compatible with your platform"))]
    IncompatiblePlatform,
}

type RecipeResult<T> = Result<T, RecipeError>;
