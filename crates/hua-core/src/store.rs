use crate::{
    components::ComponentPaths, error::*, extra, package::Package, persist::Pot, Requirement,
    UserManager,
};
use crc32fast::Hasher as Crc32Hasher;
use indexmap::IndexSet;
use rustbreak::PathDatabase;

use std::{
    collections::HashSet,
    fs::{self},
    hash::BuildHasherDefault,
    path::{Path, PathBuf},
};

/// The filename of the packages database of the store
pub const PACKAGES_DB: &str = "packages.db";

pub type Crc32BuildHasher = BuildHasherDefault<Crc32Hasher>;

/// A Store that contains all the packages installed by any user
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    database: PathDatabase<IndexSet<Package, Crc32BuildHasher>, Pot>,
    pkgs: IndexSet<Package, Crc32BuildHasher>,
    // TODO package is hashed by its name version and contents
    // But with the crc32 32-bit width packages might collide
    // with 80.000 packages probabilty of 80.000/2^32 ~ 0.002%
    // Consider other fast error detecting hashing algorithms in the future
}

impl Store {
    /// Creates a new store directory under the given path.
    /// Will return an Error if the directory already exists
    pub fn create_at_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_owned();
        fs::create_dir(&path)?;

        let hasher = Crc32BuildHasher::default();
        let database =
            PathDatabase::create_at_path(path.join(PACKAGES_DB), IndexSet::with_hasher(hasher))?;
        let pkgs = database.get_data(false)?;

        Ok(Self {
            path,
            database,
            pkgs,
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
        let pkgs = database.get_data(false)?;

        Ok(Self {
            path: path.to_owned(),
            database,
            pkgs,
        })
    }

    /// Dispatch a task over all the packages stored.
    pub fn read_packages<R, T: FnOnce(&IndexSet<Package, Crc32BuildHasher>) -> R>(
        &self,
        task: T,
    ) -> Result<R> {
        let res = self.database.read(|packages| task(&packages))?;
        Ok(res)
    }

    pub fn packages(&self) -> &IndexSet<Package, Crc32BuildHasher> {
        &self.pkgs
    }

    pub fn packages_mut(&mut self) -> &mut IndexSet<Package, Crc32BuildHasher> {
        &mut self.pkgs
    }

    // TODO return relative path for private
    // and make one public which joins self path

    /// Calculates a new path in the store for the package.
    fn get_package_path(&self, package: &Package, index: &usize) -> PathBuf {
        let name_version_hash = format!("{}-{}-{}", package.name(), package.version(), index);
        PathBuf::from(name_version_hash)
    }

    fn copy_to_store<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();
        fs::create_dir(to)?;
        extra::fs::copy_to(from, to)?;
        Ok(())
    }

    // TODO maybe do not return Err if package already inserted, but bool or enum

    /// Inserts a package into the store and returns its hash.
    pub fn insert<P: AsRef<Path>>(&mut self, package: Package, path: P) -> Result<usize> {
        let path = path.as_ref();

        let (index, _) = self.pkgs.insert_full(package);
        let package = &self.pkgs[index];
        let new_path = self.get_package_path(&package, &index);
        self.copy_to_store(path, self.path.join(new_path))?;

        Ok(index)
    }

    pub fn extend<'a, P: AsRef<Path>>(
        &'a mut self,
        packages: impl IntoIterator<Item = (Package, P)> + 'a,
    ) -> impl Iterator<Item = Result<usize>> + 'a {
        packages
            .into_iter()
            .map(|(package, path)| self.insert(package, path))
    }

    /// Links only the package at the specified path
    pub fn link_package(&self, index: usize, to: &ComponentPaths) -> Result<()> {
        let package = &self.pkgs[index];
        extra::fs::link_components(
            &self.path.join(self.get_package_path(&package, &index)),
            package.provides(),
            to,
        )?;
        Ok(())
    }

    /// Links all the packages to the specified path.
    pub fn link_packages<'a>(
        &self,
        indices: impl IntoIterator<Item = &'a usize>,
        to: &ComponentPaths,
    ) -> Result<()> {
        for index in indices {
            self.link_package(*index, to)?;
        }
        Ok(())
    }

    /// Remove all packages that are currently unused in all generations.
    pub fn remove_unused(&mut self, user_manager: &UserManager) -> Result<HashSet<u64>> {
        // let to_remove = self.database.read(|db| {
        //     let to_remove = db
        //         .nodes
        //         .iter()
        //         .filter_map(|(key, package)| {
        //             if let Ok(res) = user_manager.contains_package(key) && !res {
        //                 Some((*key, self.get_package_path(&package, key)))
        //             } else {
        //                 None
        //             }
        //         })
        //         .collect::<HashMap<u64, PathBuf>>();
        //     to_remove
        // })?;

        // // TODO before deleting all the pathes verify that packages are indeed not used by checking their relations

        // for (_key, path) in to_remove.iter() {
        //     fs::remove_dir_all(self.path.join(path))?;
        // }

        // let to_remove_keys = to_remove.into_keys().collect::<HashSet<u64>>();

        // self.database.write(|db| {
        //     for key in to_remove_keys.iter() {
        //         let res = db.nodes.remove(&key);
        //         assert!(res.is_some());

        //         let (_, idx) = db
        //             .indices
        //             .remove_by_left(&key)
        //             .expect("Every key must be in the hash-index map");
        //         let res = db.relations.remove_node(idx);
        //         assert!(res.is_some());
        //     }
        // })?;

        // Ok(to_remove_keys)
        todo!()
    }

    pub unsafe fn get_unchecked_index_of(&self, package: &Package) -> usize {
        self.pkgs.get_index_of(package).unwrap_unchecked()
    }

    pub fn get_index_of(&self, package: &Package) -> Option<usize> {
        self.pkgs.get_index_of(package)
    }

    pub fn matches<'a>(
        &'a self,
        requirement: &'a Requirement,
    ) -> impl Iterator<Item = &'a Package> {
        self.pkgs.iter().filter_map(|package| {
            if package.matches(requirement) {
                Some(package)
            } else {
                None
            }
        })
    }

    pub fn filter<P: Fn(&&Package) -> bool>(&self, predicate: P) -> impl Iterator<Item = &Package> {
        self.pkgs.iter().filter(predicate)
    }

    pub fn find_by_name_starting_with(&self, name: &str) -> Option<&Package> {
        self.find(|p| p.name().starts_with(name))
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Package> {
        self.find(|p| p.name() == name)
    }

    pub fn find<P: Fn(&&Package) -> bool>(&self, predicate: P) -> Option<&Package> {
        self.pkgs.iter().find(predicate)
    }

    pub fn get(&self, index: usize) -> Option<&Package> {
        self.pkgs.get_index(index)
    }

    pub unsafe fn get_unchecked(&self, index: usize) -> &Package {
        &self.pkgs[index]
    }

    // TODO maybe flush just after every io operation

    /// Flushes all data to the backend
    pub fn flush(self) -> Result<()> {
        self.database.put_data(self.pkgs, true)?;
        Ok(())
    }

    /// The path of the store
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::PACKAGES_DB;
    use crate::{support::*, Store};
    use std::{fs, path::Path};
    use temp_dir::TempDir;

    fn store_create_at_path(path: &Path) -> Store {
        let store = Store::create_at_path(&path).unwrap();

        let store_db = path.join(PACKAGES_DB);

        assert!(store_db.exists());
        assert!(store_db.is_file());

        store
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
        let package = pkg("package", &package_path);

        let index = store.insert(package, package_path).unwrap();

        let package = store.get(index).unwrap();
        let package_store_path = store.path().join(store.get_package_path(&package, &index));

        assert!(package_store_path.exists());
        assert!(package_store_path.is_dir());

        let package_store_file = package_store_path.join("lib/package.so");

        assert!(package_store_file.exists());
        assert!(package_store_file.is_file());
    }

    // #[test]
    // fn store_remove_unused() {
    //     let temp_dir = TempDir::new().unwrap();
    //     let path = temp_dir.child("store");
    //     let pkg_path = temp_dir.child("package");
    //     let pkg_two_path = temp_dir.child("package2");
    //     let user_manager_path = temp_dir.child("user");

    //     let mut store = store_create_at_path(&path);
    //     let pkg = package(&pkg_path, "package");
    //     let pkg_two = package(&pkg_two_path, "package2");

    //     let hash = store.insert(pkg, pkg_path).unwrap();
    //     let hash_two = store.insert(pkg_two, pkg_two_path).unwrap();

    //     let mut user_manager = UserManager::create_at_path(&user_manager_path).unwrap();
    //     assert!(user_manager.insert_package(&hash, &mut store).unwrap());

    //     let removed = store.remove_unused(&mut user_manager).unwrap();

    //     assert_eq!(removed.len(), 1);
    //     assert!(removed.get(&hash_two).is_some());
    // }

    // #[test]
    // fn store_remove_unused_empty() {
    //     let temp_dir = TempDir::new().unwrap();
    //     let path = temp_dir.child("store");
    //     let user_manager_path = temp_dir.child("user");

    //     let mut store = store_create_at_path(&path);
    //     let mut user_manager = UserManager::create_at_path(&user_manager_path).unwrap();

    //     let removed = store.remove_unused(&mut user_manager).unwrap();

    //     assert_eq!(removed.len(), 0);
    // }

    // #[test]
    // fn store_link_package() {
    //     let temp_dir = TempDir::new().unwrap();
    //     let path = temp_dir.child("store");
    //     let parent_path = temp_dir.child("parent");
    //     let child_path = temp_dir.child("child");

    //     let mut store = store_create_at_path(&path);

    //     let parent = package(&parent_path, "parent");
    //     let parent_hash = store.insert(parent, parent_path).unwrap();

    //     let child = package(&child_path, "child");
    //     let child_hash = store
    //         .add_child(&parent_hash, child, VersionReq::STAR, child_path)
    //         .unwrap();

    //     let global_temp_path = temp_dir.child("global");
    //     fs::create_dir(&global_temp_path).unwrap();
    //     let global_paths = ComponentPaths::from_path(&global_temp_path);
    //     global_paths.create_dirs().unwrap();

    //     let linked = store.link_package(&parent_hash, &global_paths).unwrap();
    //     assert_eq!(linked.len(), 2);

    //     let mut eq_linked = HashSet::new();
    //     eq_linked.insert(child_hash);
    //     eq_linked.insert(parent_hash);
    //     assert_eq!(linked, eq_linked);

    //     let child_bin_link = global_paths.binary.join("child.sh");
    //     assert!(child_bin_link.exists());
    //     assert!(child_bin_link.is_symlink());

    //     let parent_bin_link = global_paths.binary.join("parent.sh");
    //     assert!(parent_bin_link.exists());
    //     assert!(parent_bin_link.is_symlink());
    // }
}
