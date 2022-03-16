use crate::{error::*, extra, extra::ComponentPaths, package::Package, persist::Pot, UserManager};
use bimap::BiMap;
use crc32fast::Hasher as Crc32Hasher;
use daggy::{Dag, NodeIndex};
use petgraph::visit::Walker;
use rustbreak::PathDatabase;
use serde::{Deserialize, Serialize};

use std::{
    collections::{HashMap, HashSet},
    fs::{self},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

/// The filename of the packages database of the store
pub const PACKAGES_DB: &str = "packages.db";

// TODO create Store Trait, so that the local store and the cache can be used interchangable

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
struct Weight;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Packages {
    pub map: HashMap<u64, Package>,
    pub relations: Dag<Weight, Weight, usize>,
    pub hash_idx_map: BiMap<u64, NodeIndex<usize>>,
}

impl Packages {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            relations: Dag::new(),
            hash_idx_map: BiMap::new(),
        }
    }
}

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
        let path = path.as_ref();
        fs::create_dir(&path)?;

        let database = PathDatabase::create_at_path(path.join(PACKAGES_DB), Packages::new())?;

        Ok(Self {
            path: path.to_owned(),
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
        let res = self.database.read(|packages| task(&packages.map))?;
        Ok(res)
    }

    /// Hash package and update package path.
    fn hash_package(&mut self, package: &Package) -> Result<u64> {
        package.name.hash(&mut self.hasher);
        package.version.hash(&mut self.hasher);
        extra::hash_path(&package.path, &mut self.hasher)?;
        let hash = self.hasher.finish();

        Ok(hash)
    }

    /// Updates package path and returns old path.
    fn update_package_path(&self, package: &mut Package, hash: u64) -> PathBuf {
        let name_version_hash = format!("{}-{}-{}", &package.name, &package.version, hash);
        let old = std::mem::replace(&mut package.path, self.path.join(name_version_hash));
        old
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
    pub fn insert(&mut self, mut package: Package) -> Result<u64> {
        let hash = self.hash_package(&package)?;
        let old_path = self.update_package_path(&mut package, hash);

        self.database.write(|db| {
            let res = db.map.insert(hash, package);
            assert!(res.is_none());

            let idx = db.relations.add_node(Weight);

            let res = db.hash_idx_map.insert(hash, idx);
            assert!(!res.did_overwrite());
        })?;

        self.database.read(|db| -> Result<()> {
            let package = db.map.get(&hash).ok_or(Error::PackageNotFound(hash))?;
            self.copy_to_store(&old_path, &package.path, hash)?;
            Ok(())
        })??;
        Ok(hash)
    }

    /// Add a package as a child (dependency) of another package and returns its hash.
    pub fn add_child(&mut self, parent: &u64, mut package: Package) -> Result<u64> {
        let hash = self.hash_package(&package)?;
        let old_path = self.update_package_path(&mut package, hash);

        self.database.write(|db| {
            if let Some(idx) = db.hash_idx_map.get_by_left(parent) {
                if db.relations.node_weight(*idx).is_some() && db.map.contains_key(parent) {
                    let res = db.map.insert(hash, package);
                    assert!(res.is_none());

                    let (_, child_idx) = db.relations.add_child(*idx, Weight, Weight);
                    let res = db.hash_idx_map.insert(hash, child_idx);
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
            let package = db.map.get(&hash).ok_or(Error::PackageNotFound(hash))?;
            self.copy_to_store(&old_path, &package.path, hash)?;
            Ok(())
        })??;
        Ok(hash)
    }

    /// Add a package as a parent of another package(dependency) and returns its hash.
    pub fn add_parent(&mut self, child: &u64, mut package: Package) -> Result<u64> {
        let hash = self.hash_package(&mut package)?;
        let old_path = self.update_package_path(&mut package, hash);

        self.database.write(|db| {
            if let Some(idx) = db.hash_idx_map.get_by_left(child) {
                if db.relations.node_weight(*idx).is_some() && db.map.contains_key(child) {
                    let res = db.map.insert(hash, package);
                    assert!(res.is_none());

                    let (_, parent_idx) = db.relations.add_parent(*idx, Weight, Weight);
                    let res = db.hash_idx_map.insert(hash, parent_idx);
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
            let package = db.map.get(&hash).ok_or(Error::PackageNotFound(hash))?;
            self.copy_to_store(&old_path, &package.path, hash)?;
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
            let package = db.map.get(hash).ok_or(Error::PackageNotFound(*hash))?;

            extra::link_optional_component_paths(&package.optional_component_paths(), to)?;

            let package_idx = db
                .hash_idx_map
                .get_by_left(hash)
                .ok_or(Error::IndexNotFound(*hash))?;

            db.relations
                .children(*package_idx)
                .iter(&db.relations)
                .map(|(_edge, child_idx)| -> Result<()> {
                    let child = db
                        .hash_idx_map
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
    pub fn link_package(&self, hash: &u64, to: &ComponentPaths) -> Result<HashSet<u64>> {
        let mut hashes = HashSet::new();
        self.inner_link_package(hash, to, &mut hashes)?;
        Ok(hashes)
    }

    /// Links all the packages and their respective dependencies to the specified path.
    pub fn link_packages(
        &self,
        hashes: &HashSet<u64>,
        to: &ComponentPaths,
    ) -> Result<HashSet<u64>> {
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
                .map
                .iter()
                .filter_map(|(key, package)| {
                    if let Ok(res) = user_manager.contains_package(key) && !res {
                        Some((*key, package.path.clone()))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<u64, PathBuf>>();
            to_remove
        })?;

        // TODO before deleting all the pathes verify that packages are indeed not used by checking their relations

        for (_key, path) in to_remove.iter() {
            fs::remove_dir_all(path)?;
        }

        let to_remove_keys = to_remove.into_keys().collect::<HashSet<u64>>();

        self.database.write(|db| {
            for key in to_remove_keys.iter() {
                let res = db.map.remove(&key);
                assert!(res.is_some());

                let (_, idx) = db
                    .hash_idx_map
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
            db.map
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
    pub fn get_children(&self, hash: &u64) -> Result<HashSet<u64>> {
        let children = self.database.read(|db| {
            let idx = db
                .hash_idx_map
                .get_by_left(hash)
                .ok_or(Error::PackageNotFound(*hash))?;
            let children = db
                .relations
                .children(*idx)
                .iter(&db.relations)
                .map(|(_edge, node)| {
                    *db.hash_idx_map
                        .get_by_right(&node)
                        .expect("Every index must be in the hash-index map")
                })
                .collect::<HashSet<u64>>();
            Ok(children)
        })?;
        children
    }

    // TODO maybe flush just after every io operation

    /// Flushes all data to the backend
    pub fn flush(&self) -> Result<()> {
        self.database.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        fs::{self, File},
        path::Path,
    };

    use temp_dir::TempDir;

    use crate::{extra::ComponentPaths, Package, Store, UserManager};

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

        let _bin = File::create(package_bin_path.join(&format!("{}.sh", name))).unwrap();
        Package::new(name, "0.1.0", &path)
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

        let hash = store.insert(package).unwrap();

        let children = store.get_children(&hash).unwrap();

        assert!(children.is_empty());
    }

    #[test]
    fn store_add_child() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let parent_path = temp_dir.child("parent");
        let child_path = temp_dir.child("child");

        let mut store = store_create_at_path(&path);

        let parent = package(&parent_path, "parent");
        let parent_hash = store.insert(parent).unwrap();

        let child = package(&child_path, "child");
        let child_hash = store.add_child(&parent_hash, child).unwrap();

        let children = store.get_children(&parent_hash).unwrap();

        assert_eq!(children.len(), 1);
        assert!(children.get(&child_hash).is_some())
    }

    #[test]
    fn store_add_parent() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let parent_path = temp_dir.child("parent");
        let child_path = temp_dir.child("child");

        let mut store = store_create_at_path(&path);

        let child = package(&child_path, "child");
        let child_hash = store.insert(child).unwrap();

        let parent = package(&parent_path, "parent");
        let parent_hash = store.add_parent(&child_hash, parent).unwrap();

        let children = store.get_children(&parent_hash).unwrap();

        assert_eq!(children.len(), 1);
        assert!(children.get(&child_hash).is_some());
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

        let hash = store.insert(pkg).unwrap();
        let hash_two = store.insert(pkg_two).unwrap();

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
        let parent_hash = store.insert(parent).unwrap();

        let child = package(&child_path, "child");
        let child_hash = store.add_child(&parent_hash, child).unwrap();

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
