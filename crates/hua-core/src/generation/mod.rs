use snafu::prelude::*;

pub use builder::GenerationBuilder;
pub use generation::Generation;
pub use manager::GenerationManager;

mod builder;
mod generation;
mod manager;

use crate::{
    dependency::{DependencyError, Requirement},
    store::StoreError,
};

/// An [Error](std::error::Error) inside the [crate::generation] module.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum GenerationError {
    #[snafu(display("Dependency Error for generation {id}: {source}"))]
    DependencyError { id: usize, source: DependencyError },
    #[snafu(display("Missing Requirements for generation {id}"))]
    MissingRequirements { id: usize },
    #[snafu(display("Missing base path for generation {id}"))]
    MissingBasePath { id: usize },
    #[snafu(display("Missing packages for generation {id}, resolve first"))]
    MissingPackages { id: usize },
    #[snafu(display("The generation {id} is already present"))]
    AlreadyPresent { id: usize },
    #[snafu(display("The generation {id} was not found"))]
    NotFound { id: usize },
    #[snafu(display("The generation {id} is currently in use"))]
    InUse { id: usize },
    #[snafu(display("Graph not resolvable: {unresolved:?}"))]
    GraphNotResolvable { unresolved: Vec<Requirement> },
    #[snafu(display("IO Error for generation {id}: {source}"))]
    GenerationIoError { id: usize, source: std::io::Error },
    #[snafu(display("IO Error: {source}"))]
    IoError { source: std::io::Error },
    #[snafu(display("Store Error: {source}"))]
    StoreError { source: StoreError },
}

type GenerationResult<T> = Result<T, GenerationError>;
