use crate::{extra, recipe::Derivation, store::object::Blob};
use console::style;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt, fmt::Debug, hash::Hash};

/// A Requirement to be resolved by the [super::DependencyGraph].
///
/// # Example
///
/// ```
/// use std::collections::BTreeSet;
/// use semver::VersionReq;
/// use hua_core::dependency::Requirement;
/// use hua_core::store::object::Blob;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut objects = BTreeSet::new();
/// objects.insert(Blob::new("bin/bash".into()));
///
/// let requirement = Requirement::new("name".to_owned(), VersionReq::parse(">0.0.1")?, objects);
/// # Ok(())
/// # }
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct Requirement {
    name: String,
    version_req: VersionReq,
    blobs: BTreeSet<Blob>,
}

impl Requirement {
    /// Creates a new [Requirement].
    pub fn new(name: String, version_req: VersionReq, objects: BTreeSet<Blob>) -> Self {
        Self {
            name,
            version_req,
            blobs: objects,
        }
    }

    /// Returns the name of the [Requirement].
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns the [VersionReq] of the [Requirement].
    pub fn version_req(&self) -> &VersionReq {
        &self.version_req
    }

    /// Returns all the [Blob] of the [Requirement].
    pub fn blobs(&self) -> &BTreeSet<Blob> {
        &self.blobs
    }
}

// TODO: check if this Ordering does not lead to errors when deserializing from the same name and components

impl Ord for Requirement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name
            .cmp(other.name())
            .then(self.blobs.cmp(other.blobs()))
    }
}

impl PartialOrd for Requirement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}\ncomponents: {:#?}\n",
            style(&self.name).green(),
            self.version_req,
            self.blobs
        )
    }
}

impl From<(Derivation, BTreeSet<Blob>)> for Requirement {
    fn from(data: (Derivation, BTreeSet<Blob>)) -> Self {
        Self::new(
            data.0.name,
            extra::exact_version_req(data.0.version),
            data.1,
        )
    }
}
