use crate::recipe::Derivation;

use super::{
    object::{Blob, Tree},
    ObjectId, PackageId,
};
use console::style;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::{self},
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LocalPackageSource {
    pub drv: Derivation,
    pub path: PathBuf,
}

impl LocalPackageSource {
    /// Creates a new package
    pub fn new(drv: Derivation, path: PathBuf) -> Self {
        Self { drv, path }
    }

    pub fn name(&self) -> &String {
        &self.drv.name
    }

    pub fn version(&self) -> &Version {
        &self.drv.version
    }

    pub fn path_in_store<P: AsRef<Path>>(&self, store_path: P, id: &PackageId) -> PathBuf {
        self.drv.path_in_store(store_path, id)
    }
}

impl fmt::Display for LocalPackageSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Path: {:?}\nDerivation: {}", self.path, self.drv)
    }
}

pub struct RemotePackageSource {
    pub id: PackageId,
    pub drv: Derivation,
    pub base: Url,
    pub blobs: BTreeMap<Blob, ObjectId>,
    pub trees: BTreeMap<Tree, ObjectId>,
}

impl ToString for RemotePackageSource {
    fn to_string(&self) -> String {
        format!(
            "{} {}",
            style(&self.base).red(),
            style(&self.drv.name).blue()
        )
    }
}

impl RemotePackageSource {
    pub fn new(
        id: PackageId,
        drv: Derivation,
        base: Url,
        blobs: BTreeMap<Blob, ObjectId>,
        trees: BTreeMap<Tree, ObjectId>,
    ) -> Self {
        Self {
            id,
            drv,
            base,
            blobs,
            trees,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Packages {
    nodes: HashMap<PackageId, Derivation>,
    children: HashMap<PackageId, HashSet<ObjectId>>,
}

impl Packages {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains_drv(&self, drv: &Derivation) -> Option<PackageId> {
        self.nodes
            .iter()
            .find_map(|(id, other)| if drv == other { Some(*id) } else { None })
    }

    pub fn contains(&self, id: &PackageId) -> bool {
        self.nodes.contains_key(id)
    }

    pub fn insert(
        &mut self,
        id: PackageId,
        drv: Derivation,
        objects: HashSet<ObjectId>,
    ) -> Option<Derivation> {
        self.children.insert(id, objects);
        self.nodes.insert(id, drv)
    }

    pub fn insert_child(&mut self, id: &PackageId, child: ObjectId) -> Option<bool> {
        if let Some(children) = self.children.get_mut(id) {
            Some(children.insert(child))
        } else {
            None
        }
    }

    pub fn get(&self, id: &PackageId) -> Option<&Derivation> {
        self.nodes.get(id)
    }

    pub unsafe fn get_unchecked(&self, id: &PackageId) -> &Derivation {
        self.nodes.get(id).unwrap_unchecked()
    }

    pub fn get_children(&self, id: &PackageId) -> Option<&HashSet<ObjectId>> {
        self.children.get(id)
    }

    pub fn get_full(&self, id: &PackageId) -> Option<(&Derivation, &HashSet<ObjectId>)> {
        if let Some(drv) = self.nodes.get(id) {
            self.children.get(id).map(|objects| ((drv, objects)))
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: &PackageId) -> Option<(Derivation, HashSet<ObjectId>)> {
        let desc = self.nodes.remove(id);
        let children = self.children.remove(id);

        if let Some(drv) = desc && let Some(children) = children {
            Some((drv, children))
        } else {
            None
        }
    }

    pub fn path_in_store<P: AsRef<Path>>(&self, id: &PackageId, store_path: P) -> Option<PathBuf> {
        if let Some(desc) = self.get(id) {
            let name_version_id = format!("{}-{}-{}", desc.name, desc.version, id);
            Some(store_path.as_ref().join(name_version_id))
        } else {
            None
        }
    }

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

    pub fn filter<P>(
        &self,
        predicate: P,
    ) -> impl Iterator<Item = (&PackageId, &Derivation, &HashSet<ObjectId>)>
    where
        P: Fn(&PackageId, &Derivation, &HashSet<ObjectId>) -> bool,
    {
        self.nodes.iter().filter_map(move |(id, drv)| {
            let objects = unsafe { self.children.get(id).unwrap_unchecked() };
            if predicate(id, drv, objects) {
                Some((id, drv, objects))
            } else {
                None
            }
        })
    }

    pub fn filter_by_name_starting_with<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = (&PackageId, &Derivation, &HashSet<ObjectId>)> + '_ {
        self.filter(move |_id, desc, _objects| desc.name.starts_with(name))
    }

    pub fn filter_by_name_containing<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = (&PackageId, &Derivation, &HashSet<ObjectId>)> + '_ {
        self.filter(move |_id, desc, _objects| desc.name.contains(name))
    }

    pub fn find<P: Fn(&PackageId, &Derivation, &HashSet<ObjectId>) -> bool>(
        &self,
        predicate: P,
    ) -> Option<(&PackageId, &Derivation)> {
        self.nodes.iter().find(move |(id, drv)| {
            let objects = unsafe { self.children.get(id).unwrap_unchecked() };
            predicate(*id, *drv, objects)
        })
    }

    pub fn find_by_name_starting_with(&self, name: &str) -> Option<(&PackageId, &Derivation)> {
        self.find(|_id, p, _objects| p.name.starts_with(name))
    }

    pub fn find_by_name_containing(&self, name: &str) -> Option<(&PackageId, &Derivation)> {
        self.find(|_id, p, _objects| p.name.contains(name))
    }

    pub fn find_by_name(&self, name: &str) -> Option<(&PackageId, &Derivation)> {
        self.find(|_id, p, _objects| p.name == name)
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
