use semver::Version;
use serde::{Deserialize, Serialize};

use crate::extra::hash::PackageHash;
use crate::store::{Object, ObjectId};
use crate::Requirement;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use super::Blob;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
pub struct PackageDesc {
    pub name: String,
    pub version: Version,
    pub blobs: BTreeSet<Blob>,
    pub requires: HashSet<Requirement>,
}

impl PackageDesc {
    pub fn new(
        name: String,
        version: Version,
        blobs: BTreeSet<Blob>,
        requires: HashSet<Requirement>,
    ) -> Self {
        Self {
            name,
            version,
            blobs,
            requires,
        }
    }

    pub fn matches(&self, requirement: &Requirement) -> bool {
        self.blobs.is_superset(requirement.blobs())
            && requirement.name() == &self.name
            && requirement.version_req().matches(&self.version)
    }
}

impl From<Package> for PackageDesc {
    fn from(package: Package) -> Self {
        let blobs = package
            .provides
            .into_iter()
            .filter_map(|(_id, obj)| Object::into_blob(obj))
            .collect();

        Self {
            name: package.name,
            version: package.version,
            blobs,
            requires: package.requires,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Package {
    pub id: ObjectId,
    pub name: String,
    pub version: Version,
    pub provides: HashMap<ObjectId, Object>,
    pub requires: HashSet<Requirement>,
}

impl Package {
    /// Creates a new package
    pub fn new(
        id: ObjectId,
        name: &str,
        version: Version,
        provides: HashMap<ObjectId, Object>,
        requires: HashSet<Requirement>,
    ) -> Self {
        Self {
            id,
            name: name.to_owned(),
            version,
            provides,
            requires,
        }
    }

    pub fn id(&self) -> ObjectId {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn provides(&self) -> &HashMap<ObjectId, Object> {
        &self.provides
    }

    pub fn requires(&self) -> &HashSet<Requirement> {
        &self.requires
    }

    pub fn verify<P: AsRef<Path>>(&self, path: P) -> io::Result<bool> {
        let PackageHash { root, children: _ } = PackageHash::from_path(path, &self.name)?;

        Ok(self.id == root)
    }

    pub fn path_in_store<P: AsRef<Path>>(&self, store_path: P) -> PathBuf {
        let name_version_id = format!("{}-{}-{}", self.name, self.version, self.id);
        store_path.as_ref().join(name_version_id)
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}-{}", self.name, self.version))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Packages {
    nodes: HashMap<ObjectId, PackageDesc>,
}

impl Packages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains_package(&self, package: &Package) -> bool {
        self.nodes.contains_key(&package.id)
    }

    pub fn contains(&self, id: &ObjectId) -> bool {
        self.nodes.contains_key(id)
    }

    pub fn insert(&mut self, id: ObjectId, desc: PackageDesc) -> Option<PackageDesc> {
        self.nodes.insert(id, desc)
    }

    pub fn get(&self, id: &ObjectId) -> Option<&PackageDesc> {
        self.nodes.get(id)
    }

    pub unsafe fn get_unchecked(&self, id: &ObjectId) -> &PackageDesc {
        self.nodes.get(id).unwrap_unchecked()
    }

    pub fn matches<'a>(
        &'a self,
        requirement: &'a Requirement,
    ) -> impl Iterator<Item = (&ObjectId, &'a PackageDesc)> {
        self.nodes.iter().filter_map(|(id, desc)| {
            if desc.matches(requirement) {
                Some((id, desc))
            } else {
                None
            }
        })
    }

    pub fn path_in_store<P: AsRef<Path>>(&self, id: &ObjectId, store_path: P) -> Option<PathBuf> {
        if let Some(desc) = self.get(id) {
            let name_version_id = format!("{}-{}-{}", desc.name, desc.version, id);
            Some(store_path.as_ref().join(name_version_id))
        } else {
            None
        }
    }

    pub fn filter<P: Fn(&PackageDesc) -> bool>(
        &self,
        predicate: P,
    ) -> impl Iterator<Item = (&ObjectId, &PackageDesc)> {
        self.nodes
            .iter()
            .filter(move |(_id, desc)| predicate(*desc))
    }

    pub fn find_by_name_starting_with(&self, name: &str) -> Option<(&ObjectId, &PackageDesc)> {
        self.find(|p| p.name.starts_with(name))
    }

    pub fn find_by_name(&self, name: &str) -> Option<(&ObjectId, &PackageDesc)> {
        self.find(|p| p.name == name)
    }

    pub fn find<P: Fn(&PackageDesc) -> bool>(
        &self,
        predicate: P,
    ) -> Option<(&ObjectId, &PackageDesc)> {
        self.nodes.iter().find(move |(_id, desc)| predicate(*desc))
    }
}
