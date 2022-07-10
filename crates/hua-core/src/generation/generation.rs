use crate::{dependency::Requirement, extra::path::ComponentPathBuf, store::id::PackageId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::{
    fmt,
    path::{Path, PathBuf},
};

// TODO: Instead of storing the requirements as Requirement, store them as packages,
// so that upgrading can be provided

/// A Generation which has different [packages](crate::store::Package) linked
/// and keeps track of the [requirements](Requirement) it fullfills.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Generation {
    path: PathBuf,
    packages: HashSet<PackageId>,
    requirements: HashSet<Requirement>,
    component_paths: ComponentPathBuf,
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Path: {:#?}\nNumber of packages: {}",
            self.path,
            self.packages.len()
        )
    }
}

impl Generation {
    /// Create a new [Generation].
    pub fn new(
        path: PathBuf,
        packages: HashSet<PackageId>,
        requirements: HashSet<Requirement>,
        component_paths: ComponentPathBuf,
    ) -> Self {
        Self {
            path,
            packages,
            requirements,
            component_paths,
        }
    }

    /// List all [package ids](PackageId) inside.
    #[deprecated]
    pub fn list_packages(&self) {
        println!("{:#?}", self.packages);
    }

    /// Get all [package ids](PackageId) inside the [Generation].
    pub fn packages(&self) -> &HashSet<PackageId> {
        &self.packages
    }

    /// Checks if a [PackageId] is insde the [Generation].
    pub fn contains(&self, id: &PackageId) -> bool {
        self.packages.contains(id)
    }

    /// Returns the [Path], where all [packages](crate::store::Package) are linked
    /// into the specific [ComponentPathBuf].
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the [ComponentPathBuf], where all [packages](crate::store::Package) are linked.
    pub fn component_paths(&self) -> &ComponentPathBuf {
        &self.component_paths
    }

    /// The [requirements](Requirement) the [Generation] fulfills.
    pub fn requirements(&self) -> &HashSet<Requirement> {
        &self.requirements
    }

    /// Checks if the [Requirement] is fulfilled by the [Generation].
    pub fn contains_requirement(&self, requirement: &Requirement) -> bool {
        self.requirements.contains(requirement)
    }
}
