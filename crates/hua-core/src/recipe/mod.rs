use snafu::prelude::*;

pub use recipe::Recipe;

use crate::GenerationError;

mod recipe;

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
    #[snafu(display("TomlSerilizationError: {source}"))]
    TomlSerilizationError { source: toml::ser::Error },
}

type RecipeResult<T> = Result<T, RecipeError>;

pub const LINUX: u8 = 0x01;

pub const X86_64: u8 = 0x01;
pub const X86: u8 = 0x02;

fn check_archs(archs: u8) -> RecipeResult<()> {
    if cfg!(target_arch = "x86_64") && archs & X86_64 != X86_64 {
        Err(RecipeError::IncompatibleArchitecture)
    } else if cfg!(target_arch = "x86") && archs & X86 != X86 {
        Err(RecipeError::IncompatibleArchitecture)
    } else {
        Ok(())
    }
}

fn check_platforms(platforms: u8) -> RecipeResult<()> {
    if cfg!(target_os = "linux") && platforms & LINUX != LINUX {
        Err(RecipeError::IncompatiblePlatform)
    } else {
        Ok(())
    }
}