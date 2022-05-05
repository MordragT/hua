use crate::store::object::Blob;
use snafu::prelude::*;
use std::{fmt::Debug, hash::Hash};

mod dependency_graph;
mod requirement;
mod step;

pub use dependency_graph::DependencyGraph;
pub use requirement::Requirement;

#[derive(Debug, PartialEq, Eq, Hash)]
enum Conflict<'a> {
    Name(&'a String),
    Blob(&'a Blob),
}

/// All Errors that occur in this module
#[allow(missing_docs)]
#[derive(Debug, Snafu, PartialEq, Eq)]
pub enum DependencyError {
    /// A cycle was detected
    #[snafu(display("Cycle detected: {error}"))]
    CycleDetected { error: String },
    /// Found a conflicting name
    #[snafu(display("Found conflicting name {name}"))]
    ConflictingName { name: String },
    /// Found a conflicting blob
    #[snafu(display("Found conflicting blob {blob:#?}"))]
    ConflictingBlob { blob: Blob },
}

impl<'a> From<Conflict<'a>> for DependencyError {
    fn from(conflict: Conflict<'a>) -> Self {
        match conflict {
            Conflict::Blob(b) => Self::ConflictingBlob { blob: b.to_owned() },
            Conflict::Name(n) => Self::ConflictingName { name: n.to_owned() },
        }
    }
}

type DependencyResult<T> = std::result::Result<T, DependencyError>;
