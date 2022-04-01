use crate::store::Blob;
use snafu::prelude::*;
use std::{fmt::Debug, hash::Hash};

mod dependency_graph;
mod requirement;
mod step;

pub use dependency_graph::DependencyGraph;
pub use requirement::Requirement;
pub use step::Step;

// TODO: Tests

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Conflict<'a> {
    Name(&'a String),
    Blob(&'a Blob),
}

#[derive(Debug, Snafu, PartialEq, Eq)]
pub enum DependencyError {
    #[snafu(display("Cycle detected: {error}"))]
    CycleDetected { error: String },
    #[snafu(display("Found conflicting name {name}"))]
    ConflictingName { name: String },
    #[snafu(display("Found conflicting blob {blob:#?}"))]
    ConflictingBlob { blob: Blob },
    #[snafu(display("Requirements not resolvable"))]
    RequirementsNotResolvable,
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
