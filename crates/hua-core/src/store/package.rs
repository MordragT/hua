use super::{
    object::{Blob, Tree},
    ObjectId, PackageId,
};
use crate::{dependency::Requirement, extra::hash::PackageHash};
use console::style;
use log::debug;
use relative_path::RelativePathBuf;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::{self},
    io,
    path::{Path, PathBuf},
};
use url::Url;

/// A description to a [Package].
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
pub struct PackageDesc {
    /// The name of the [Package].
    pub name: String,
    /// The description of a [Package].
    pub desc: String,
    /// The [Version] of a [Package].
    pub version: Version,
    /// A collection of licenses (allows dual-licensing).
    pub licenses: Vec<String>,
    /// A collection of [requirements](Requirement) the [Package] needs.
    pub requires: HashSet<Requirement>,
}

impl PackageDesc {
    /// Create a new [PackageDesc].
    pub fn new(
        name: String,
        desc: String,
        version: Version,
        licenses: Vec<String>,
        requires: HashSet<Requirement>,
    ) -> Self {
        Self {
            name,
            desc,
            version,
            licenses,
            requires,
        }
    }
}

impl fmt::Display for PackageDesc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}\n\t{}\n",
            style(&self.name).green(),
            &self.version,
            &self.desc,
        )?;
        f.write_str("\tlicenses: ")?;
        for license in &self.licenses {
            write!(f, "{license} ")?;
        }
        f.write_str("\n\trequires:\n")?;
        for req in &self.requires {
            write!(f, "{req}\n")?;
        }
        Ok(())
    }
}

/// A Package which can be inserted into a [Store](super::Store).
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Package {
    /// The calculated [PackageId].
    /// Uniquely identifying the [Package].
    #[serde_as(as = "DisplayFromStr")]
    pub id: PackageId,
    /// The description of the [Package].
    pub desc: PackageDesc,
}

impl Package {
    /// Creates a new package.
    pub fn new(id: PackageId, desc: PackageDesc) -> Self {
        Self { id, desc }
    }

    /// Returns the name of a [Package].
    pub fn name(&self) -> &String {
        &self.desc.name
    }

    /// Returns the [Version] of a [Package].
    pub fn version(&self) -> &Version {
        &self.desc.version
    }

    /// Returns the [requirements](Requirement), which need to be fulfilled for the [Package].
    pub fn requires(&self) -> &HashSet<Requirement> {
        &self.desc.requires
    }

    /// Checks if the given [PackageId] can be calculated from the given [Path].
    pub fn verify<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> io::Result<(bool, BTreeMap<Tree, ObjectId>, BTreeMap<Blob, ObjectId>)> {
        let PackageHash { root, trees, blobs } = PackageHash::from_path(path, &self.desc.name)?;
        debug!("Calculated root {root}");

        Ok((self.id == root, trees, blobs))
    }

    /// Caclulates the [PathBuf] of the [Package] with the given store path.
    pub fn path_in_store<P: AsRef<Path>>(&self, store_path: P) -> PathBuf {
        let name_version_id = format!("{}-{}-{}", self.desc.name, self.desc.version, self.id);
        store_path.as_ref().join(name_version_id)
    }

    /// Returns the [RelativePathBuf] of the [Package].
    pub fn relative_path(&self) -> RelativePathBuf {
        format!("{}-{}-{}", self.desc.name, self.desc.version, self.id).into()
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x} {}", style(self.id.truncate()).blue(), self.desc)
    }
}

impl AsRef<PackageDesc> for Package {
    fn as_ref(&self) -> &PackageDesc {
        &self.desc
    }
}

/// Allows Operations on multiple [packages](Package).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Packages {
    nodes: HashMap<PackageId, PackageDesc>,
    children: HashMap<PackageId, HashSet<ObjectId>>,
}

impl Packages {
    /// Creates a new [Packages] object.
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(missing_docs)]
    #[deprecated(note = "Use `contains(&package.id)` instead")]
    pub fn contains_package(&self, package: &Package) -> bool {
        self.nodes.contains_key(&package.id)
    }

    /// Checks if the [PackageId] is present.
    pub fn contains(&self, id: &PackageId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Inserts a [Package] and its [ObjectId] children,
    /// which can retrieve a [Object](super::Object) with the [Objects](super::object::Objects) data structure.
    pub fn insert(
        &mut self,
        id: PackageId,
        desc: PackageDesc,
        objects: HashSet<ObjectId>,
    ) -> Option<PackageDesc> {
        self.children.insert(id, objects);
        self.nodes.insert(id, desc)
    }

    /// Inserts a single [ObjectId] as child into the specified [Package].
    pub fn insert_child(&mut self, id: &PackageId, child: ObjectId) -> Option<bool> {
        if let Some(children) = self.children.get_mut(id) {
            Some(children.insert(child))
        } else {
            None
        }
    }

    /// Get a single [PackageDesc].
    pub fn get(&self, id: &PackageId) -> Option<&PackageDesc> {
        self.nodes.get(id)
    }

    /// Get a single [PackageDesc] unchecked.
    pub unsafe fn get_unchecked(&self, id: &PackageId) -> &PackageDesc {
        self.nodes.get(id).unwrap_unchecked()
    }

    /// Get all [children](ObjectId) of the [PackageId].
    pub fn get_children(&self, id: &PackageId) -> Option<&HashSet<ObjectId>> {
        self.children.get(id)
    }

    /// Get the [PackageDesc] aswell as the [children](ObjectId) of the [PackageId].
    pub fn get_full(&self, id: &PackageId) -> Option<(&PackageDesc, &HashSet<ObjectId>)> {
        if let Some(desc) = self.nodes.get(id) {
            self.children.get(id).map(|objects| ((desc, objects)))
        } else {
            None
        }
    }

    /// Removes the [PackageId] and returns the corresponding [PackageDesc] and [children](ObjectId) if found.
    pub fn remove(&mut self, id: &PackageId) -> Option<(PackageDesc, HashSet<ObjectId>)> {
        let desc = self.nodes.remove(id);
        let children = self.children.remove(id);

        if let Some(desc) = desc && let Some(children) = children {
            Some((desc, children))
        } else {
            None
        }
    }

    /// Calculates the [PathBuf] inside the [Store's](super::Store) path, if the [PackageDesc] is found by the [PackageId].
    pub fn path_in_store<P: AsRef<Path>>(&self, id: &PackageId, store_path: P) -> Option<PathBuf> {
        if let Some(desc) = self.get(id) {
            let name_version_id = format!("{}-{}-{}", desc.name, desc.version, id);
            Some(store_path.as_ref().join(name_version_id))
        } else {
            None
        }
    }

    /// Calculates the [Url] inside the [Store's](super::Store) url, if the [PackageDesc] is found by the [PackageId].
    pub fn url_in_store(&self, id: &PackageId, store_url: &Url) -> Option<Url> {
        if let Some(desc) = self.get(id) {
            let name_version_id = format!("{}-{}-{}/", desc.name, desc.version, id);
            Some(
                store_url
                    .join(&name_version_id)
                    .expect("INTERNAL ERROR: The package part of an url must be parseable"),
            )
        } else {
            None
        }
    }

    /// Filters all packages by the specified `predicate`.
    pub fn filter<P>(
        &self,
        predicate: P,
    ) -> impl Iterator<Item = (&PackageId, &PackageDesc, &HashSet<ObjectId>)>
    where
        P: Fn(&PackageId, &PackageDesc, &HashSet<ObjectId>) -> bool,
    {
        self.nodes.iter().filter_map(move |(id, desc)| {
            let objects = unsafe { self.children.get(id).unwrap_unchecked() };
            if predicate(id, desc, objects) {
                Some((id, desc, objects))
            } else {
                None
            }
        })
    }

    /// Filters all packages by the specified `name`.
    pub fn filter_by_name_starting_with<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = (&PackageId, &PackageDesc, &HashSet<ObjectId>)> + '_ {
        self.filter(move |_id, desc, _objects| desc.name.starts_with(name))
    }

    /// Filters all packages by the specified `name`.
    pub fn filter_by_name_containing<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = (&PackageId, &PackageDesc, &HashSet<ObjectId>)> + '_ {
        self.filter(move |_id, desc, _objects| desc.name.contains(name))
    }

    /// Finds a package by the specified `predicate`.
    pub fn find<P: Fn(&PackageId, &PackageDesc, &HashSet<ObjectId>) -> bool>(
        &self,
        predicate: P,
    ) -> Option<(&PackageId, &PackageDesc)> {
        self.nodes.iter().find(move |(id, desc)| {
            let objects = unsafe { self.children.get(id).unwrap_unchecked() };
            predicate(*id, *desc, objects)
        })
    }

    /// Finds a package by the specified `name`.
    pub fn find_by_name_starting_with(&self, name: &str) -> Option<(&PackageId, &PackageDesc)> {
        self.find(|_id, p, _objects| p.name.starts_with(name))
    }

    /// Finds a package by the specified `name`.
    pub fn find_by_name_containing(&self, name: &str) -> Option<(&PackageId, &PackageDesc)> {
        self.find(|_id, p, _objects| p.name.contains(name))
    }

    /// Finds a package by the specified `name`.
    pub fn find_by_name(&self, name: &str) -> Option<(&PackageId, &PackageDesc)> {
        self.find(|_id, p, _objects| p.name == name)
    }

    /// Finds a package by the specified [ObjectId].
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
