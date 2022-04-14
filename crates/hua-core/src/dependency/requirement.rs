use crate::{
    extra,
    store::{object::Blob, package::PackageDesc},
};
use console::style;
use semver::VersionReq;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fmt, fmt::Debug, hash::Hash};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct Requirement {
    name: String,
    version_req: VersionReq,
    blobs: BTreeSet<Blob>,
}

impl Requirement {
    pub fn new(name: String, version_req: VersionReq, objects: BTreeSet<Blob>) -> Self {
        Self {
            name,
            version_req,
            blobs: objects,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn version_req(&self) -> &VersionReq {
        &self.version_req
    }
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

impl From<(PackageDesc, BTreeSet<Blob>)> for Requirement {
    fn from(data: (PackageDesc, BTreeSet<Blob>)) -> Self {
        Self::new(
            data.0.name,
            extra::exact_version_req(data.0.version),
            data.1,
        )
    }
}
