use crate::{store::PackageId, Requirement};
use std::{collections::HashSet, fmt::Debug, hint::unreachable_unchecked};

#[derive(Debug, PartialEq, Eq)]
pub enum Step<'a> {
    Resolved(PackageId),
    Choice(HashSet<PackageId>),
    Unresolved(&'a Requirement),
}

impl<'a> Step<'a> {
    pub unsafe fn as_choice_unchecked(&self) -> &HashSet<PackageId> {
        match &self {
            Self::Choice(set) => set,
            _ => unreachable_unchecked(),
        }
    }

    pub unsafe fn as_resolved_unchecked(&self) -> PackageId {
        match self {
            Self::Resolved(id) => *id,
            _ => unreachable_unchecked(),
        }
    }

    pub unsafe fn as_unresolved_unchecked(&self) -> &'a Requirement {
        match &self {
            Self::Unresolved(req) => req,
            _ => unreachable_unchecked(),
        }
    }

    pub fn as_resolved(&self) -> Option<PackageId> {
        match self {
            Step::Resolved(id) => Some(*id),
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
