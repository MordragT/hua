use semver::Version;
use serde::{Deserialize, Serialize};

use crate::extra::collections::OrdValTreeMap;
use crate::extra::hash::PackageHash;
use crate::store::ObjectId;
use crate::Requirement;
use serde_with::{serde_as, DisplayFromStr};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use super::{Blob, PackageId, Tree};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
pub struct PackageDesc {
    pub name: String,
    pub desc: String,
    pub version: Version,
    pub licenses: Vec<String>,
    // TODO change to HashSet of ObjectIds
    pub blobs: BTreeSet<Blob>,
    pub requires: HashSet<Requirement>,
}

impl PackageDesc {
    pub fn new(
        name: String,
        desc: String,
        version: Version,
        licenses: Vec<String>,
        blobs: BTreeSet<Blob>,
        requires: HashSet<Requirement>,
    ) -> Self {
        Self {
            name,
            desc,
            version,
            licenses,
            blobs,
            requires,
        }
    }

    pub fn matches(&self, requirement: &Requirement) -> bool {
        self.blobs.is_superset(requirement.blobs())
            && requirement.name() == &self.name
            && requirement.version_req().matches(&self.version)
    }

    pub fn from_package(package: Package, blobs: impl IntoIterator<Item = Blob>) -> Self {
        let blobs = blobs.into_iter().collect();

        Self {
            name: package.name,
            desc: package.desc,
            version: package.version,
            licenses: package.licenses,
            blobs,
            requires: package.requires,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Package {
    #[serde_as(as = "DisplayFromStr")]
    pub id: PackageId,
    pub name: String,
    pub desc: String,
    pub version: Version,
    pub licenses: Vec<String>,
    pub requires: HashSet<Requirement>,
    // #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    // pub trees: BTreeMap<ObjectId, Tree>,
    // #[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
    // pub blobs: BTreeMap<ObjectId, Blob>,
}

impl Package {
    /// Creates a new package
    pub fn new(
        id: PackageId,
        name: String,
        desc: String,
        version: Version,
        licenses: Vec<String>,
        // trees: BTreeMap<ObjectId, Tree>,
        // blobs: BTreeMap<ObjectId, Blob>,
        requires: HashSet<Requirement>,
    ) -> Self {
        Self {
            id,
            name,
            desc,
            version,
            licenses,
            // trees,
            // blobs,
            requires,
        }
    }

    // pub fn id(&self) -> PackageId {
    //     self.id
    // }

    // pub fn name(&self) -> &String {
    //     &self.name
    // }

    // pub fn version(&self) -> &Version {
    //     &self.version
    // }

    // pub fn requires(&self) -> &HashSet<Requirement> {
    //     &self.requires
    // }

    pub fn verify<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> io::Result<(bool, OrdValTreeMap<ObjectId, Tree>, HashMap<ObjectId, Blob>)> {
        let PackageHash { root, trees, blobs } = PackageHash::from_path(path, &self.name)?;

        Ok((self.id == root, trees, blobs))
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
    nodes: HashMap<PackageId, PackageDesc>,
    children: HashMap<PackageId, HashSet<ObjectId>>,
}

impl Packages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains_package(&self, package: &Package) -> bool {
        self.nodes.contains_key(&package.id)
    }

    pub fn contains(&self, id: &PackageId) -> bool {
        self.nodes.contains_key(id)
    }

    pub fn insert(
        &mut self,
        id: PackageId,
        desc: PackageDesc,
        objects: HashSet<ObjectId>,
    ) -> Option<PackageDesc> {
        self.children.insert(id, objects);
        self.nodes.insert(id, desc)
    }

    pub fn insert_child(&mut self, id: &PackageId, child: ObjectId) -> Option<bool> {
        if let Some(children) = self.children.get_mut(id) {
            Some(children.insert(child))
        } else {
            None
        }
    }

    pub fn get(&self, id: &PackageId) -> Option<&PackageDesc> {
        self.nodes.get(id)
    }

    pub unsafe fn get_unchecked(&self, id: &PackageId) -> &PackageDesc {
        self.nodes.get(id).unwrap_unchecked()
    }

    pub fn get_children(&self, id: &PackageId) -> Option<&HashSet<ObjectId>> {
        self.children.get(id)
    }

    pub fn find_children_by_requirement(
        &self,
        requirement: &Requirement,
    ) -> Option<&HashSet<ObjectId>> {
        if let Some((id, _desc)) = self.find_by_requirement(requirement) {
            let children = unsafe { self.get_children(id).unwrap_unchecked() };
            Some(children)
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: &PackageId) -> Option<(PackageDesc, HashSet<ObjectId>)> {
        let desc = self.nodes.remove(id);
        let children = self.children.remove(id);

        if let Some(desc) = desc && let Some(children) = children {
            Some((desc, children))
        } else {
            None
        }
    }

    pub fn matches<'a>(
        &'a self,
        requirement: &'a Requirement,
    ) -> impl Iterator<Item = (&PackageId, &'a PackageDesc)> {
        self.nodes
            .iter()
            .filter(|(_id, desc)| desc.matches(requirement))
    }

    pub fn path_in_store<P: AsRef<Path>>(&self, id: &PackageId, store_path: P) -> Option<PathBuf> {
        if let Some(desc) = self.get(id) {
            let name_version_id = format!("{}-{}-{}", desc.name, desc.version, id);
            Some(store_path.as_ref().join(name_version_id))
        } else {
            None
        }
    }

    pub fn filter<P: Fn(&PackageId, &PackageDesc) -> bool>(
        &self,
        predicate: P,
    ) -> impl Iterator<Item = (&PackageId, &PackageDesc)> {
        self.nodes
            .iter()
            .filter(move |(id, desc)| predicate(*id, *desc))
    }

    pub fn find_by_name_starting_with(&self, name: &str) -> Option<(&PackageId, &PackageDesc)> {
        self.find(|_id, p| p.name.starts_with(name))
    }

    pub fn find_by_name(&self, name: &str) -> Option<(&PackageId, &PackageDesc)> {
        self.find(|_id, p| p.name == name)
    }

    pub fn find_by_requirement(
        &self,
        requirement: &Requirement,
    ) -> Option<(&PackageId, &PackageDesc)> {
        self.nodes
            .iter()
            .find(|(_id, desc)| desc.matches(requirement))
    }

    pub fn find<P: Fn(&PackageId, &PackageDesc) -> bool>(
        &self,
        predicate: P,
    ) -> Option<(&PackageId, &PackageDesc)> {
        self.nodes
            .iter()
            .find(move |(id, desc)| predicate(*id, *desc))
    }

    pub fn find_package_id(&self, object_id: &ObjectId) -> Option<&PackageId> {
        self.children.iter().find_map(|(package_id, set)| {
            if set.contains(object_id) {
                Some(package_id)
            } else {
                None
            }
        })
    }
}
