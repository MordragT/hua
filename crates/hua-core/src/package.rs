use semver::Version;

use crate::{extra, Component, Requirement};
use std::collections::BTreeSet;
use std::fmt;
use std::hash::Hash;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Hash, PartialEq, Eq)]
pub struct Package {
    name: String,
    version: Version,
    provides: BTreeSet<Component>,
    requires: BTreeSet<Requirement>,
}

impl Package {
    /// Creates a new package
    pub fn new(
        name: &str,
        version: Version,
        provides: BTreeSet<Component>,
        requires: BTreeSet<Requirement>,
    ) -> Self {
        Self {
            name: name.to_owned(),
            version,
            provides,
            requires,
        }
    }

    /// Creates a dependency from a package
    pub fn into_requirement(self) -> Requirement {
        Requirement::new(
            self.name,
            extra::exact_version_req(self.version),
            self.provides,
        )
    }

    pub fn matches(&self, requirement: &Requirement) -> bool {
        requirement.components().is_subset(&self.provides)
            && requirement.name() == &self.name
            && requirement.version_req().matches(&self.version)
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn provides(&self) -> &BTreeSet<Component> {
        &self.provides
    }

    pub fn requires(&self) -> &BTreeSet<Requirement> {
        &self.requires
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}-{}", self.name, self.version))
    }
}
