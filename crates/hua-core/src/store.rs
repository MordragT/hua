use crate::{error::*, extra, package::Package, persist::Pot, ComponentPaths, UserManager};
use bimap::BiMap;
use crc32fast::Hasher as Crc32Hasher;
use daggy::{Dag, NodeIndex};
use petgraph::graphmap::DiGraphMap;
use petgraph::visit::Walker;
use rustbreak::PathDatabase;
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    fs::{self},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    sync::RwLockReadGuard,
};

const PACKAGES_DB: &str = "packages.db";

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

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    database: PathDatabase<Packages, Pot>,
    // packages: PathDatabase<HashMap<u64, Package>, Pot>,
    // relations: PathDatabase<DiGraphMap<u64, Weight>, Pot>,
    // TODO package is hashed by its name version and contents
    // But with the crc32 32-bit width packages might collide
    // with 80.000 packages probabilty of 80.000/2^32 ~ 0.002%
    // Consider other fast error detecting hashing algorithms in the future
    hasher: Crc32Hasher,
}

impl Store {
    /// Creates a new store directory under the given path
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

    /// Opens a store under the specified path
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

    pub fn read<R, T: FnOnce(&HashMap<u64, Package>) -> R>(&self, task: T) -> Result<R> {
        let res = self.database.read(|packages| task(&packages.map))?;
        Ok(res)
    }

    /// Hash package and update package path
    fn hash_package(&mut self, package: &Package) -> Result<u64> {
        package.name.hash(&mut self.hasher);
        package.version.hash(&mut self.hasher);
        extra::hash_path(&package.path, &mut self.hasher)?;
        let hash = self.hasher.finish();

        Ok(hash)
    }

    /// Updates package path and returns old path
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
    pub fn insert(&mut self, mut package: Package) -> Result<()> {
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
        })?;
        Ok(())
    }

    pub fn add_child(&mut self, parent: &u64, mut package: Package) -> Result<()> {
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
        })?;

        self.database.read(|db| -> Result<()> {
            let package = db.map.get(&hash).ok_or(Error::PackageNotFound(hash))?;
            self.copy_to_store(&old_path, &package.path, hash)?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn add_parent(&mut self, child: &u64, mut package: Package) -> Result<()> {
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
        })?;

        self.database.read(|db| -> Result<()> {
            let package = db.map.get(&hash).ok_or(Error::PackageNotFound(hash))?;
            self.copy_to_store(&old_path, &package.path, hash)?;
            Ok(())
        })?;
        Ok(())
    }

    // TODO very slow benchmark

    fn inner_link_package(
        &self,
        hash: &u64,
        to: &ComponentPaths,
        hashes: &mut Vec<u64>,
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
        })?;

        hashes.push(*hash);
        Ok(())
    }

    /// Links the packakge and all its dependencies to the specified path
    /// Returns a list of all package hashes linked the process
    pub fn link_package(&self, hash: &u64, to: &ComponentPaths) -> Result<Vec<u64>> {
        let mut hashes = Vec::new();
        self.inner_link_package(hash, to, &mut hashes)?;
        Ok(hashes)
    }

    /// Links all the packages and their respective dependencies to the specified path
    pub fn link_packages(&self, hashes: &[u64], to: &ComponentPaths) -> Result<Vec<u64>> {
        let mut res = Vec::new();
        hashes
            .iter()
            .map(|hash| {
                self.inner_link_package(hash, to, &mut res)?;
                Ok(())
            })
            .collect::<Result<()>>()?;
        Ok(res)
    }

    pub fn remove_unused(&mut self, user_manager: &UserManager) -> Result<()> {
        let to_remove = self.database.read(|db| {
            let to_remove = db
                .map
                .iter()
                .filter_map(|(key, package)| {
                    if user_manager
                        .packages()
                        .find(|installed| *installed == key)
                        .is_none()
                    {
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

        let to_remove_keys = to_remove.into_keys();

        self.database.write(|db| {
            for key in to_remove_keys {
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

        Ok(())
    }
}
