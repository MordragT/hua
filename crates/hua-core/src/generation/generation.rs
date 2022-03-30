use crate::components::ComponentPaths;
use crate::Requirement;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Generation {
    path: PathBuf,
    packages: HashSet<usize>,
    requirements: HashSet<Requirement>,
    component_paths: ComponentPaths,
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
        packages: HashSet<usize>,
        requirements: HashSet<Requirement>,
        component_paths: ComponentPaths,
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

    pub fn packages(&self) -> &HashSet<usize> {
        &self.packages
    }

    pub fn contains(&self, index: &usize) -> bool {
        self.packages.contains(index)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn component_paths(&self) -> &ComponentPaths {
        &self.component_paths
    }

    pub fn requirements(&self) -> &HashSet<Requirement> {
        &self.requirements
    }

    pub fn contains_requirement(&self, requirement: &Requirement) -> bool {
        self.requirements.contains(requirement)
    }
}
