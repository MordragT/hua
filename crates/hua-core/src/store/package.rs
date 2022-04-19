use super::{
    object::{Blob, Tree},
    ObjectId, PackageId,
};
use crate::{dependency::Requirement, extra::hash::PackageHash};
use console::style;
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

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
pub struct PackageDesc {
    pub name: String,
    pub desc: String,
    pub version: Version,
    pub licenses: Vec<String>,
    pub requires: HashSet<Requirement>,
}

impl PackageDesc {
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

#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Package {
    #[serde_as(as = "DisplayFromStr")]
    pub id: PackageId,
    pub desc: PackageDesc,
}

impl Package {
    /// Creates a new package
    pub fn new(id: PackageId, desc: PackageDesc) -> Self {
        Self { id, desc }
    }

    pub fn name(&self) -> &String {
        &self.desc.name
    }

    pub fn version(&self) -> &Version {
        &self.desc.version
    }

    pub fn requires(&self) -> &HashSet<Requirement> {
        &self.desc.requires
    }

    pub fn verify<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> io::Result<(bool, BTreeMap<Tree, ObjectId>, BTreeMap<Blob, ObjectId>)> {
        let PackageHash { root, trees, blobs } = PackageHash::from_path(path, &self.desc.name)?;

        Ok((self.id == root, trees, blobs))
    }

    pub fn path_in_store<P: AsRef<Path>>(&self, store_path: P) -> PathBuf {
        let name_version_id = format!("{}-{}-{}", self.desc.name, self.desc.version, self.id);
        store_path.as_ref().join(name_version_id)
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

    pub fn get_full(&self, id: &PackageId) -> Option<(&PackageDesc, &HashSet<ObjectId>)> {
        if let Some(desc) = self.nodes.get(id) {
            self.children.get(id).map(|objects| ((desc, objects)))
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
            let name_version_id = format!("{}-{}-{}", desc.name, desc.version, id);
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

    pub fn filter_by_name_starting_with<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = (&PackageId, &PackageDesc, &HashSet<ObjectId>)> + '_ {
        self.filter(move |_id, desc, _objects| desc.name.starts_with(name))
    }

    pub fn find<P: Fn(&PackageId, &PackageDesc, &HashSet<ObjectId>) -> bool>(
        &self,
        predicate: P,
    ) -> Option<(&PackageId, &PackageDesc)> {
        self.nodes.iter().find(move |(id, desc)| {
            let objects = unsafe { self.children.get(id).unwrap_unchecked() };
            predicate(*id, *desc, objects)
        })
    }

    pub fn find_by_name_starting_with(&self, name: &str) -> Option<(&PackageId, &PackageDesc)> {
        self.find(|_id, p, _objects| p.name.starts_with(name))
    }

    pub fn find_by_name(&self, name: &str) -> Option<(&PackageId, &PackageDesc)> {
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
