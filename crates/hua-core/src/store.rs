use crate::{
    components::ComponentPaths, dependency::Conflicts, error::*, extra, package::Package,
    persist::Pot, Component, Requirement, UserManager,
};
use bimap::BiMap;
use crc32fast::Hasher as Crc32Hasher;
use daggy::{Dag, NodeIndex};
use petgraph::visit::Walker;
use rustbreak::PathDatabase;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs::{self},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

/// The filename of the packages database of the store
pub const PACKAGES_DB: &str = "packages.db";

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct Packages {
//     nodes: HashMap<u64, Package>,
//     relations: Dag<Version, Requirement, usize>,
//     indices: BiMap<u64, NodeIndex<usize>>,
//     provided_by: HashMap<Requirement, HashSet<u64>>,
// }

// impl Packages {
//     pub fn new() -> Self {
//         Self {
//             nodes: HashMap::new(),
//             relations: Dag::new(),
//             indices: BiMap::new(),
//             provided_by: HashMap::new(),
//         }
//     }

//     pub fn search(&self, name: &str) -> Option<&Package> {
//         self.nodes.iter().find_map(|(_, package)| {
//             if package.name.starts_with(name) {
//                 Some(package)
//             } else {
//                 None
//             }
//         })
//     }

//     // pub fn remove(&mut self, hash: &u64) -> Option<Package> {
//     //     self.nodes.remove(hash)
//     // }

//     pub fn get(&self, hash: &u64) -> Option<&Package> {
//         self.nodes.get(hash)
//     }

//     // pub fn get_children(&self, hash: &u64) -> Option<HashSet<u64>> {
//     //     if let Some(idx) = self.indices.get_by_left(hash) {
//     //         let children = self
//     //             .relations
//     //             .children(*idx)
//     //             .iter(&self.relations)
//     //             .map(|(_edge, node)| {
//     //                 *self
//     //                     .indices
//     //                     .get_by_right(&node)
//     //                     .expect("Every index must be in the hash-index map")
//     //             })
//     //             .collect::<HashSet<u64>>();

//     //         Some(children)
//     //     } else {
//     //         None
//     //     }
//     // }

//     /// Get all requirements of an package
//     pub fn get_requirements(&self, hash: &u64) -> Option<impl Iterator<Item = &Requirement>> {
//         if let Some(index) = self.indices.get_by_left(hash) {
//             let requirements =
//                 self.relations
//                     .children(*index)
//                     .iter(&self.relations)
//                     .map(|(edge, _node)| {
//                         self.relations
//                             .edge_weight(edge)
//                             .expect("INTERNAL: Must be present.")
//                     });
//             Some(requirements)
//         } else {
//             None
//         }
//     }

//     // pub fn resolve_requirements(&self, requirements: &HashSet<u64>) -> Result<HashSet<u64>> {
//     //     let mut collector = HashSet::new();
//     //     for hash in packages {
//     //         self.inner_resolve_depdendency(hash, &mut collector)?;
//     //     }
//     //     Ok(collector)
//     // }

//     // If same requirements but different dependency in store -> use more recent version of dependency
//     // If two or more requirements have conflicting names or conflicting components -> check if one dependency can resolve all of them

//     pub fn resolve_requirements(
//         &self,
//         packages: impl Iterator<Item = u64>,
//     ) -> Result<HashSet<u64>> {
//         // Does graph.children traverse all children to the leaf or just the direct children ?

//         struct ConflictMap {}

//         #[derive(Debug)]
//         struct Dependencies<'a> {
//             options: HashMap<&'a Requirement, HashSet<u64>>,
//             names: HashMap<&'a String, &'a Requirement>,
//             components: HashMap<&'a Component, &'a Requirement>,
//             conflicts: HashMap<&'a Requirement, HashMap<Conflict<'a>, &'a Requirement>>,
//         }

//         impl<'a> Dependencies<'a> {
//             pub fn new() -> Self {
//                 Self {
//                     options: HashMap::new(),
//                     names: HashMap::new(),
//                     components: HashMap::new(),
//                     conflicts: HashMap::new(),
//                 }
//             }

//             pub fn add_conflict(
//                 &mut self,
//                 conflict: Conflict<'a>,
//                 req: &'a Requirement,
//                 other: &'a Requirement,
//             ) {
//                 if let Some(map) = self.conflicts.get_mut(req) {
//                     map.insert(conflict, other);
//                 } else {
//                     let mut map = HashMap::new();
//                     map.insert(conflict, other);

//                     self.conflicts.insert(req, map);
//                 }
//             }
//         }

//         let mut deps = Dependencies::new();

//         for hash in packages {
//             let index = self
//                 .indices
//                 .get_by_left(&hash)
//                 .ok_or(Error::IndexNotFound(hash))?;
//             for (edge, node) in self.relations.children(*index).iter(&self.relations) {
//                 let req = self
//                     .relations
//                     .edge_weight(edge)
//                     .expect("INTERNAL: Must be present.");

//                 let hash = self
//                     .indices
//                     .get_by_right(&node)
//                     .expect("INTERNAL: Must be present.");

//                 if let Some(set) = deps.options.get_mut(req) {
//                     set.insert(*hash);
//                 } else {
//                     let mut set = HashSet::new();
//                     set.insert(*hash);

//                     deps.options.insert(req, set);
//                 }

//                 if let Some(other) = deps.names.get_mut(req.name()) && other != &req {
//                     let conflict = Conflict::Name(req.name());
//                     deps.add_conflict(conflict, req, other);
//                     *other = req;
//                 } else {
//                     deps.names.insert(req.name(), req);
//                 }

//                 for component in req.components() {
//                     if let Some(other) = deps.components.get_mut(component) && other != &req {
//                         let conflict = Conflict::Component(component);
//                         deps.add_conflict(conflict, req, other);
//                         *other = req;
//                     } else {
//                         deps.components.insert(component, req);
//                     }
//                 }
//             }
//         }

//         let mut result = HashSet::new();

//         for (req, conflicts) in deps.conflicts {}

//         'names: for (name, set) in names {
//             if set.len() > 1 {
//                 // check if components are equal
//                 // amd then if there is a package in options that matches all version requirements
//                 // put it in result
//                 // remove requirement from options
//                 let mut components: HashSet<Component> = HashSet::new();
//                 let mut version_reqs = HashSet::new();

//                 for req in set {
//                     components.extend(req.components().clone());
//                     version_reqs.insert(req.version_req());
//                 }

//                 for req in set {
//                     options.remove(req);

//                     'hashes: for hash in options.get(req).expect("INTERNAL: Must be present.") {
//                         let package = self.nodes.get(hash).expect("INTERNAL: Must be present.");
//                         if package.provides.is_superset(&components) {
//                             for version_req in version_reqs {
//                                 if !version_req.matches(&package.version) {
//                                     continue 'hashes;
//                                 }
//                             }
//                             result.insert(*hash);
//                             continue 'names;
//                         }
//                     }
//                 }
//                 return Err(Error::RequirementNameCollision(name.to_owned()));
//             }
//         }

//         for (component, set) in components {
//             if set.len() > 1 {
//                 // check if names are equal
//                 // amd then if there is a package in options that matches all version requirements
//                 // put it in result
//                 // remove requirement from options

//                 let mut name = None;
//                 let mut version_reqs = HashSet::new();

//                 for req in set {
//                     if let Some(name) = name && name != req.name() {
//                         return Err(Error::RequirementNameCollision(name.to_owned()));
//                     } else {
//                         name = Some(req.name());
//                     }
//                     version_reqs.insert(req.version_req());
//                 }

//                 for req in set {
//                     options.remove(req);
//                 }
//             }
//         }

//         for (_req, set) in options {
//             let hash = set.drain().next().expect("INTERNAL: Must be present.");
//             result.insert(hash);
//         }

//         Ok(result)
//     }

//     fn gather_requirements<'a, 'b>(
//         &'a self,
//         hash: &'b u64,
//         collector: &'a mut HashMap<&'a Requirement, HashSet<u64>>,
//     ) {
//         if let Some(idx) = self.indices.get_by_left(&hash) {
//             for (req, hash) in
//                 self.relations
//                     .children(*idx)
//                     .iter(&self.relations)
//                     .map(|(edge, node)| {
//                         let req = self
//                             .relations
//                             .edge_weight(edge)
//                             .expect("INTERNAL: Must be present.");
//                         let hash = self
//                             .indices
//                             .get_by_right(&node)
//                             .expect("INTERNAL: Must be present.");
//                         (req, hash)
//                     })
//             {
//                 self.gather_requirements(hash, collector);
//                 if let Some(set) = collector.get_mut(req) {
//                     set.insert(*hash);
//                 } else {
//                     let mut set = HashSet::new();
//                     set.insert(*hash);
//                     collector.insert(req, set);
//                 }
//             }
//         }
//     }

//     // TODO will probably overflow with many dependencies, put depdencies in state ?

//     // fn inner_get_depdendencies(
//     //     &self,
//     //     hash: &u64,
//     //     collector: &mut HashSet<Box<dyn Dependency>>,
//     // ) -> Result<()> {
//     //     let idx = self
//     //         .hash_idx_map
//     //         .get_by_left(hash)
//     //         .ok_or(Error::PackageNotFound(*hash))?;
//     //     for (edge, node) in self.relations.children(*idx).iter(&self.relations) {
//     //         let version_req = self
//     //             .relations
//     //             .edge_weight(edge)
//     //             .expect("Edge must be present.");
//     //         let hash = self
//     //             .hash_idx_map
//     //             .get_by_right(&node)
//     //             .expect("Every index must be in the hash-index map");
//     //         let package = self.map.get(hash).expect("Must be present");
//     //         let dependency = package.into_dependency(*version_req, self.get_dependencies(&hash)?);
//     //         collector.insert(dependency);
//     //     }
//     //     Ok(())
//     // }
// }

/// A Store that contains all the packages installed by any user
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    database: PathDatabase<Packages, Pot>,
    // TODO package is hashed by its name version and contents
    // But with the crc32 32-bit width packages might collide
    // with 80.000 packages probabilty of 80.000/2^32 ~ 0.002%
    // Consider other fast error detecting hashing algorithms in the future
    hasher: Crc32Hasher,
}

impl Store {
    /// Creates a new store directory under the given path.
    /// Will return an Error if the directory already exists
    pub fn create_at_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_owned();
        fs::create_dir(&path)?;

        let database = PathDatabase::create_at_path(path.join(PACKAGES_DB), Packages::new())?;

        Ok(Self {
            path,
            database,
            hasher: Crc32Hasher::new(),
        })
    }

    /// Opens a store under the specified path.
    /// Returns an error if the path does not exists or
    /// does not contain the necessary files
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::PathNotFound(path.to_owned()));
        }

        let database = PathDatabase::load_from_path(path.join(PACKAGES_DB))?;

        Ok(Self {
            path: path.to_owned(),
            hasher: Crc32Hasher::new(),
            database,
        })
    }

    /// Dispatch a task over all the packages stored.
    pub fn read_packages<R, T: FnOnce(&HashMap<u64, Package>) -> R>(&self, task: T) -> Result<R> {
        let res = self.database.read(|packages| task(&packages.nodes))?;
        Ok(res)
    }

    // TODO package should already be hashed in the build step by the recipe

    /// Hash package and update package path.
    fn hash_package(&mut self, package: &Package, path: &Path) -> Result<u64> {
        package.name.hash(&mut self.hasher);
        package.version.hash(&mut self.hasher);
        extra::hash_path(path, &mut self.hasher)?;
        let hash = self.hasher.finish();

        Ok(hash)
    }

    /// Updates package path and returns old path.
    // fn update_package_path(&self, package: &mut Package, hash: u64) -> PathBuf {
    //     let name_version_hash = format!("{}-{}-{}", &package.name, &package.version, hash);
    //     let old = std::mem::replace(&mut package.path, PathBuf::from(name_version_hash));
    //     old
    // }

    /// Calculates a new path in the store for the package.
    pub fn get_package_path(&self, package: &Package, hash: &u64) -> PathBuf {
        let name_version_hash = format!("{}-{}-{}", &package.name, &package.version, hash);
        PathBuf::from(name_version_hash)
    }

    fn copy_to_store<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: P,
        to: Q,
        hash: u64,
    ) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();

        if !to.exists() {
            fs::create_dir(to)?;

            extra::copy_to(from, to)?;
            Ok(())
        } else {
            Err(Error::PackageAlreadyPresent(hash))
        }
    }

    // TODO maybe do not return Err if package already inserted, but bool or enum

    /// Inserts a package into the store and returns its hash.
    pub fn insert<P: AsRef<Path>>(&mut self, package: Package, path: P) -> Result<u64> {
        let path = path.as_ref();
        let hash = self.hash_package(&package, path)?;
        let new_path = self.get_package_path(&package, &hash);

        self.database.write(|db| {
            let idx = db.relations.add_node(package.version.clone());

            let res = db.nodes.insert(hash, package);
            assert!(res.is_none());

            let res = db.indices.insert(hash, idx);
            assert!(!res.did_overwrite());
        })?;

        self.database.read(|db| -> Result<()> {
            let package = db.nodes.get(&hash).ok_or(Error::PackageNotFound(hash))?;
            self.copy_to_store(path, self.path.join(new_path), hash)?;
            Ok(())
        })??;
        Ok(hash)
    }

    /// Add a package as a child (dependency) of another package and returns its hash.
    pub fn add_child<P: AsRef<Path>>(
        &mut self,
        parent: &u64,
        package: Package,
        version_req: VersionReq,
        path: P,
    ) -> Result<u64> {
        if !version_req.matches(&package.version) {
            return Err(Error::VersionMissmatch((package.version, version_req)));
        }

        let path = path.as_ref();
        let hash = self.hash_package(&package, path)?;
        let new_path = self.get_package_path(&package, &hash);

        self.database.write(|db| {
            if let Some(idx) = db.indices.get_by_left(parent) {
                if db.relations.node_weight(*idx).is_some() && db.nodes.contains_key(parent) {
                    let requirement = package.clone().into_requirement(version_req);

                    if let Some(set) = db.provided_by.get_mut(&requirement) {
                        set.insert(hash);
                    } else {
                        let mut set = HashSet::new();
                        set.insert(hash);
                        db.provided_by.insert(requirement.clone(), set);
                    }

                    let (_, child_idx) =
                        db.relations
                            .add_child(*idx, requirement, package.version.clone());

                    let res = db.nodes.insert(hash, package);
                    assert!(res.is_none());

                    let res = db.indices.insert(hash, child_idx);
                    assert!(!res.did_overwrite());

                    Ok(())
                } else {
                    Err(Error::PackageNotFound(*parent))
                }
            } else {
                Err(Error::IndexNotFound(*parent))
            }
        })??;

        self.database.read(|db| -> Result<()> {
            let package = db.nodes.get(&hash).ok_or(Error::PackageNotFound(hash))?;
            self.copy_to_store(path, self.path.join(new_path), hash)?;
            Ok(())
        })??;
        Ok(hash)
    }

    /// Add a package as a parent of another package(dependency) and returns its hash.
    pub fn add_parent<P: AsRef<Path>>(
        &mut self,
        child: &u64,
        package: Package,
        version_req: VersionReq,
        path: P,
    ) -> Result<u64> {
        if !version_req.matches(&package.version) {
            return Err(Error::VersionMissmatch((package.version, version_req)));
        }

        let path = path.as_ref();
        let hash = self.hash_package(&package, path)?;
        let new_path = self.get_package_path(&package, &hash);

        self.database.write(|db| {
            if let Some(idx) = db.indices.get_by_left(child) {
                if db.relations.node_weight(*idx).is_some() && db.nodes.contains_key(child) {
                    let requirement = package.clone().into_requirement(version_req);

                    if let Some(set) = db.provided_by.get_mut(&requirement) {
                        set.insert(hash);
                    } else {
                        let mut set = HashSet::new();
                        set.insert(hash);
                        db.provided_by.insert(requirement.clone(), set);
                    }

                    let (_, parent_idx) =
                        db.relations
                            .add_parent(*idx, requirement, package.version.clone());

                    let res = db.nodes.insert(hash, package.clone());
                    assert!(res.is_none());

                    let res = db.indices.insert(hash, parent_idx);
                    assert!(!res.did_overwrite());

                    Ok(())
                } else {
                    Err(Error::PackageNotFound(*child))
                }
            } else {
                Err(Error::IndexNotFound(*child))
            }
        })??;

        self.database.read(|db| -> Result<()> {
            let package = db.nodes.get(&hash).ok_or(Error::PackageNotFound(hash))?;
            self.copy_to_store(path, self.path.join(new_path), hash)?;
            Ok(())
        })??;
        Ok(hash)
    }

    // TODO very slow benchmark

    fn inner_link_package(
        &self,
        hash: &u64,
        to: &ComponentPaths,
        hashes: &mut HashSet<u64>,
    ) -> Result<()> {
        self.database.read(|db| -> Result<()> {
            let package = db.nodes.get(hash).ok_or(Error::PackageNotFound(*hash))?;

            extra::link_components(&self.path, &package.provides, to)?;

            let package_idx = db
                .indices
                .get_by_left(hash)
                .ok_or(Error::IndexNotFound(*hash))?;

            db.relations
                .children(*package_idx)
                .iter(&db.relations)
                .map(|(_edge, child_idx)| -> Result<()> {
                    let child = db
                        .indices
                        .get_by_right(&child_idx)
                        .ok_or(Error::HashNotFound(child_idx))?;
                    self.inner_link_package(child, to, hashes)?;
                    Ok(())
                })
                .collect::<Result<()>>()?;
            Ok(())
        })??;

        hashes.insert(*hash);
        Ok(())
    }

    /// Links the packakge and all its dependencies to the specified path.
    /// Returns a list of all package hashes linked the process.
    fn link_package(&self, hash: &u64, to: &ComponentPaths) -> Result<HashSet<u64>> {
        let mut hashes = HashSet::new();
        self.inner_link_package(hash, to, &mut hashes)?;
        Ok(hashes)
    }

    /// Links all the packages and their respective dependencies to the specified path.
    fn link_packages(&self, hashes: &HashSet<u64>, to: &ComponentPaths) -> Result<HashSet<u64>> {
        let mut res = HashSet::new();
        hashes
            .iter()
            .map(|hash| {
                self.inner_link_package(hash, to, &mut res)?;
                Ok(())
            })
            .collect::<Result<()>>()?;
        Ok(res)
    }

    /// Remove all packages that are currently unused in all generations.
    pub fn remove_unused(&mut self, user_manager: &UserManager) -> Result<HashSet<u64>> {
        let to_remove = self.database.read(|db| {
            let to_remove = db
                .nodes
                .iter()
                .filter_map(|(key, package)| {
                    if let Ok(res) = user_manager.contains_package(key) && !res {
                        Some((*key, self.get_package_path(&package, key)))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<u64, PathBuf>>();
            to_remove
        })?;

        // TODO before deleting all the pathes verify that packages are indeed not used by checking their relations

        for (_key, path) in to_remove.iter() {
            fs::remove_dir_all(self.path.join(path))?;
        }

        let to_remove_keys = to_remove.into_keys().collect::<HashSet<u64>>();

        self.database.write(|db| {
            for key in to_remove_keys.iter() {
                let res = db.nodes.remove(&key);
                assert!(res.is_some());

                let (_, idx) = db
                    .indices
                    .remove_by_left(&key)
                    .expect("Every key must be in the hash-index map");
                let res = db.relations.remove_node(idx);
                assert!(res.is_some());
            }
        })?;

        Ok(to_remove_keys)
    }

    /// Searches the store for the given name and do a specified task on them.
    pub fn search<R, T: Fn(&u64, &Package) -> R>(&self, name: &str, task: T) -> Result<Vec<R>> {
        let res = self.database.read(|db| {
            db.nodes
                .iter()
                .filter_map(|(key, package)| {
                    if package.name.starts_with(name) {
                        Some(task(key, package))
                    } else {
                        None
                    }
                })
                .collect::<Vec<R>>()
        })?;
        Ok(res)
    }

    /// Searches for the children of the specified package
    // pub fn get_children(&self, hash: &u64) -> Result<HashSet<u64>> {
    //     let children = self.database.read(|db| db.get_children(hash))?;
    //     children.ok_or(Error::PackageNotFound(*hash))
    // }

    // TODO maybe flush just after every io operation

    /// Flushes all data to the backend
    pub fn flush(&self) -> Result<()> {
        self.database.save()?;
        Ok(())
    }

    /// The path of the store
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        fs::{self, File},
        path::Path,
    };

    use relative_path::RelativePathBuf;
    use semver::{Version, VersionReq};
    use temp_dir::TempDir;

    use crate::{components::ComponentPaths, Binary, Component, Package, Store, UserManager};

    use super::PACKAGES_DB;

    fn store_create_at_path(path: &Path) -> Store {
        let store = Store::create_at_path(&path).unwrap();

        let store_db = path.join(PACKAGES_DB);

        assert!(store_db.exists());
        assert!(store_db.is_file());

        store
    }

    fn package(path: &Path, name: &str) -> Package {
        let package_bin_path = path.join("bin");
        fs::create_dir_all(&package_bin_path).unwrap();

        let shell_name = format!("{}.sh", name);
        let shell_path = package_bin_path.join(&shell_name);

        let _bin = File::create(&shell_path).unwrap();
        let mut provides = HashSet::new();
        provides.insert(Component::Binary(Binary::Shell(RelativePathBuf::from(
            &format!("bin/{}", shell_name),
        ))));

        Package::new(name, Version::new(1, 0, 0), provides)
    }

    #[test]
    fn store_create_at_path_ok() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let _store = store_create_at_path(&path);
    }

    #[test]
    fn store_create_at_path_err() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        fs::create_dir(&path).unwrap();

        let res = Store::create_at_path(&path);

        assert!(res.is_err());
    }

    #[test]
    fn store_open_ok() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let _store = store_create_at_path(&path);

        let _store = Store::open(&path).unwrap();
    }

    #[test]
    fn store_open_err() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");

        let res = Store::open(&path);

        assert!(res.is_err());
    }

    #[test]
    fn store_insert() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let package_path = temp_dir.child("package");

        let mut store = store_create_at_path(&path);
        let package = package(&package_path, "package");

        let hash = store.insert(package, package_path).unwrap();

        let children = store.get_children(&hash).unwrap();
        assert!(children.is_empty());

        store
            .read_packages(|pkgs| {
                let package = pkgs.get(&hash).unwrap();
                let package_store_path = store.get_package_path(&package, &hash);
                assert!(package_store_path.exists());
                assert!(package_store_path.is_dir());
            })
            .unwrap();
    }

    #[test]
    fn store_add_child() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let parent_path = temp_dir.child("parent");
        let child_path = temp_dir.child("child");

        let mut store = store_create_at_path(&path);

        let parent = package(&parent_path, "parent");
        let parent_hash = store.insert(parent, parent_path).unwrap();

        let child = package(&child_path, "child");
        let child_hash = store
            .add_child(&parent_hash, child, VersionReq::STAR, child_path)
            .unwrap();

        let children = store.get_children(&parent_hash).unwrap();

        assert_eq!(children.len(), 1);
        assert!(children.get(&child_hash).is_some());

        store
            .read_packages(|pkgs| {
                let child = pkgs.get(&child_hash).unwrap();
                let child_store_path = store.get_package_path(&child, &child_hash);
                assert!(child_store_path.exists());
                assert!(child_store_path.is_dir());

                let parent = pkgs.get(&parent_hash).unwrap();
                let parent_store_path = store.get_package_path(&parent, &parent_hash);
                assert!(parent_store_path.exists());
                assert!(parent_store_path.is_dir());
            })
            .unwrap();
    }

    #[test]
    fn store_add_parent() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let parent_path = temp_dir.child("parent");
        let child_path = temp_dir.child("child");

        let mut store = store_create_at_path(&path);

        let child = package(&child_path, "child");
        let child_hash = store.insert(child, child_path).unwrap();

        let parent = package(&parent_path, "parent");
        let parent_hash = store
            .add_parent(&child_hash, parent, VersionReq::STAR, parent_path)
            .unwrap();

        let children = store.get_children(&parent_hash).unwrap();

        assert_eq!(children.len(), 1);
        assert!(children.get(&child_hash).is_some());

        store
            .read_packages(|pkgs| {
                let child = pkgs.get(&child_hash).unwrap();
                let child_store_path = store.get_package_path(&child, &child_hash);
                assert!(child_store_path.exists());
                assert!(child_store_path.is_dir());

                let parent = pkgs.get(&parent_hash).unwrap();
                let parent_store_path = store.get_package_path(&parent, &parent_hash);
                assert!(parent_store_path.exists());
                assert!(parent_store_path.is_dir());
            })
            .unwrap();
    }

    #[test]
    fn store_remove_unused() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let pkg_path = temp_dir.child("package");
        let pkg_two_path = temp_dir.child("package2");
        let user_manager_path = temp_dir.child("user");

        let mut store = store_create_at_path(&path);
        let pkg = package(&pkg_path, "package");
        let pkg_two = package(&pkg_two_path, "package2");

        let hash = store.insert(pkg, pkg_path).unwrap();
        let hash_two = store.insert(pkg_two, pkg_two_path).unwrap();

        let mut user_manager = UserManager::create_at_path(&user_manager_path).unwrap();
        assert!(user_manager.insert_package(&hash, &mut store).unwrap());

        let removed = store.remove_unused(&mut user_manager).unwrap();

        assert_eq!(removed.len(), 1);
        assert!(removed.get(&hash_two).is_some());
    }

    #[test]
    fn store_remove_unused_empty() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let user_manager_path = temp_dir.child("user");

        let mut store = store_create_at_path(&path);
        let mut user_manager = UserManager::create_at_path(&user_manager_path).unwrap();

        let removed = store.remove_unused(&mut user_manager).unwrap();

        assert_eq!(removed.len(), 0);
    }

    #[test]
    fn store_link_package() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let parent_path = temp_dir.child("parent");
        let child_path = temp_dir.child("child");

        let mut store = store_create_at_path(&path);

        let parent = package(&parent_path, "parent");
        let parent_hash = store.insert(parent, parent_path).unwrap();

        let child = package(&child_path, "child");
        let child_hash = store
            .add_child(&parent_hash, child, VersionReq::STAR, child_path)
            .unwrap();

        let global_temp_path = temp_dir.child("global");
        fs::create_dir(&global_temp_path).unwrap();
        let global_paths = ComponentPaths::from_path(&global_temp_path);
        global_paths.create_dirs().unwrap();

        let linked = store.link_package(&parent_hash, &global_paths).unwrap();
        assert_eq!(linked.len(), 2);

        let mut eq_linked = HashSet::new();
        eq_linked.insert(child_hash);
        eq_linked.insert(parent_hash);
        assert_eq!(linked, eq_linked);

        let child_bin_link = global_paths.binary.join("child.sh");
        assert!(child_bin_link.exists());
        assert!(child_bin_link.is_symlink());

        let parent_bin_link = global_paths.binary.join("parent.sh");
        assert!(parent_bin_link.exists());
        assert!(parent_bin_link.is_symlink());
    }
}
