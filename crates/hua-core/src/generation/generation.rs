use crate::{dependency::Requirement, extra::path::ComponentPathBuf, store::id::PackageId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::{
    fmt,
    path::{Path, PathBuf},
};

// TODO: Instead of storing the requirements as Requirement, store them as packages,
// so that upgrading can be provided

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Generation {
    path: PathBuf,
    packages: HashSet<PackageId>,
    requirements: HashSet<Requirement>,
    component_paths: ComponentPathBuf,
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "Path: {:#?}\nNumber of packages: {}",
            self.path,
            self.packages.len()
        ))
    }
}

impl Generation {
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

    pub fn list_packages(&self) {
        println!("{:#?}", self.packages);
    }

    pub fn packages(&self) -> &HashSet<PackageId> {
        &self.packages
    }

    pub fn contains(&self, id: &PackageId) -> bool {
        self.packages.contains(id)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn component_paths(&self) -> &ComponentPathBuf {
        &self.component_paths
    }

    pub fn requirements(&self) -> &HashSet<Requirement> {
        &self.requirements
    }

    pub fn contains_requirement(&self, requirement: &Requirement) -> bool {
        self.requirements.contains(requirement)
    }
}
