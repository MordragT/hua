use crate::{Component, Package};
use std::{collections::HashSet, fmt::Debug, hash::Hash};

mod dependency_graph;
mod error;
mod requirement;

pub use dependency_graph::DependencyGraph;
pub use error::DependencyError;
pub use requirement::Requirement;

// TODO: Tests

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Conflict<'a> {
    Name(&'a String),
    Component(&'a Component),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Step<'a> {
    Resolved(usize),
    Choice(HashSet<&'a Package>),
    Unresolved(&'a Requirement),
}

impl<'a> Step<'a> {
    pub unsafe fn as_choice_unchecked(&self) -> &HashSet<&'a Package> {
        match &self {
            Self::Choice(set) => set,
            _ => unreachable!(),
        }
    }

    pub fn as_resolved(&self) -> Option<usize> {
        match self {
            Step::Resolved(index) => Some(*index),
            _ => None,
        }
    }

    pub fn is_resolved(&self) -> bool {
        if let Step::Resolved(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_unresolved(&self) -> Option<&Requirement> {
        match self {
            Step::Unresolved(req) => Some(req),
            _ => None,
        }
    }

    pub fn is_unresolved(&self) -> bool {
        if let Step::Unresolved(_) = self {
            true
        } else {
            false
        }
    }
}
