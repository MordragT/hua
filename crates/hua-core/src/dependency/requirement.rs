use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt, fmt::Debug, hash::Hash};

use crate::Component;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct Requirement {
    name: String,
    version_req: VersionReq,
    components: BTreeSet<Component>,
}

impl Requirement {
    pub fn new(name: String, version_req: VersionReq, components: BTreeSet<Component>) -> Self {
        Self {
            name,
            version_req,
            components,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn version_req(&self) -> &VersionReq {
        &self.version_req
    }
    pub fn components(&self) -> &BTreeSet<Component> {
        &self.components
    }
}

// TODO: check if this Ordering does not lead to errors when deserializing from the same name and components

impl Ord for Requirement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name
            .cmp(other.name())
            .then(self.components.cmp(other.components()))
    }
}

impl PartialOrd for Requirement {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "Dependency {}:\nversion_req: {}\ncomponents: {:#?}\n",
            self.name, self.version_req, self.components
        ))
    }
}
