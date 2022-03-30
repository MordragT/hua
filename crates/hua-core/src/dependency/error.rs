use snafu::prelude::*;

use crate::{Component, Conflict};

#[derive(Debug, Snafu)]
pub enum DependencyError {
    #[snafu(display("Cycle detected: {error}"))]
    CycleDetected { error: String },
    #[snafu(display("Found conflicting name {name}"))]
    ConflictingName { name: String },
    #[snafu(display("Found conflicting names"))]
    ConflictingNames,
    #[snafu(display("Found conflicting component {component}"))]
    ConflictingComponent { component: Component },
    #[snafu(display("Found conflicting components"))]
    ConflictingComponents,
}

pub type DependencyResult<T> = std::result::Result<T, DependencyError>;

impl<'a> From<Conflict<'a>> for DependencyError {
    fn from(conflict: Conflict<'a>) -> Self {
        match conflict {
            Conflict::Component(c) => Self::ConflictingComponent {
                component: c.to_owned(),
            },
            Conflict::Name(n) => Self::ConflictingName { name: n.to_owned() },
        }
    }
}
