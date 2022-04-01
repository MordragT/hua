use crate::{
    extra::{self, hash::Blake3, path::ComponentPaths},
    Requirement, UserManager,
};
use rs_merkle::MerkleProof;
use rustbreak::PathDatabase;
use std::{
    collections::{BTreeSet, HashSet},
    fs::{self},
    os::unix,
    path::{Path, PathBuf},
};

use super::*;

/// The filename of the packages database of the store
pub const PACKAGES_DB: &str = "packages.db";

/// A Store that contains all the packages installed by any user
/// Content Addressable Store
#[derive(Debug)]
pub struct Store<B: Backend> {
    path: PathBuf,
    backend: B,
}

impl<B: Backend<Source = PathBuf>> Store<B> {
    /// Creates a new store directory under the given path.
    /// Will return an Error if the directory already exists
    pub fn init<P: AsRef<Path>>(path: P) -> StoreResult<Self> {
        let path = path.as_ref().to_owned();
        fs::create_dir(&path).context(IoSnafu)?;

        let backend = B::init(path.join(PACKAGES_DB))?;

        Ok(Self { path, backend })
    }

    /// Opens a store under the specified path.
    /// Returns an error if the path does not exists or
    /// does not contain the necessary files
    pub fn open<P: AsRef<Path>>(path: P) -> StoreResult<Self> {
        let path = path.as_ref().to_owned();

        if !path.exists() {
            return Err(StoreError::NotExisting { path });
        }

        let backend = B::open(path.join(PACKAGES_DB))?;

        Ok(Self { path, backend })
    }
}

impl<B: Backend> Store<B> {
    pub fn packages(&self) -> &Packages {
        self.backend.packages()
    }

    pub fn packages_mut(&mut self) -> &mut Packages {
        self.backend.packages_mut()
    }

    pub fn objects(&self) -> &Objects {
        self.backend.objects()
    }

    pub fn objects_mut(&mut self) -> &mut Objects {
        self.backend.objects_mut()
    }

    // TODO return relative path for private
    // and make one public which joins self path

    /// Calculates a new path in the store for the package.
    fn get_package_path(&self, package: &Package, index: &usize) -> PathBuf {
        let name_version_hash = format!("{}-{}-{}", package.name(), package.version(), index);
        PathBuf::from(name_version_hash)
    }

    fn copy_to_store<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> StoreResult<()> {
        let from = from.as_ref();
        let to = to.as_ref();
        fs::create_dir(to).context(IoSnafu)?;
        extra::fs::copy_to(from, to).context(IoSnafu)?;
        Ok(())
    }

    // fn object_path_in_store(&self, package_id: &ObjectId, object_id: &ObjectId) -> Option<PathBuf> {
    //     if let Some(parent) = self.packages().path_in_store(package_id, &self.path) && let Some(obj) = self.objects().get(object_id) {
    //         Some(obj.to_path(parent))
    //     } else {
    //         None
    //     }
    // }

    fn get_full_object_path(&self, object_id: &ObjectId) -> Option<PathBuf> {
        if let Some((obj, ext)) = self.objects().get_full(&object_id) {
            let package_path = self.packages().path_in_store(&ext.package_id, &self.path);

            package_path.map(|path| obj.to_path(path))
        } else {
            None
        }
    }

    // TODO maybe do not return Err if package already inserted, but bool or enum

    /// Inserts a package into the store and returns true if the package was not present and was inserted
    /// and false if it was already present.
    pub fn insert<P: AsRef<Path>>(&mut self, package: Package, path: P) -> StoreResult<bool> {
        if !package.verify(&path).context(IoSnafu)? {
            return Err(StoreError::PackageNotVerified { package });
        }

        let path_in_store = package.path_in_store(self.path());
        fs::create_dir(&path_in_store).context(IoSnafu)?;

        let Package {
            id: package_id,
            name,
            version,
            requires,
            provides,
        } = package;

        let mut blobs = BTreeSet::new();

        if self.packages().contains(&package_id) {
            Ok(false)
        } else {
            for (id, object) in provides {
                let obj_dest = object.to_path(&path_in_store);

                if let Object::Blob(blob) = &object {
                    blobs.insert(blob.clone());
                }

                if self.objects().contains(&id) {
                    let from = self.get_full_object_path(&id).unwrap();

                    if obj_dest.exists() {
                        if obj_dest.is_dir() {
                            fs::remove_dir_all(&obj_dest).context(IoSnafu)?;
                        } else {
                            fs::remove_file(&obj_dest).context(IoSnafu)?;
                        }
                    }
                    unix::fs::symlink(from, &obj_dest).context(IoSnafu)?;

                    assert!(self.objects_mut().insert_link(&id, obj_dest));
                } else {
                    let obj_src = object.to_path(&path);

                    match object.kind() {
                        ObjectKind::Blob => {
                            if let Some(parent) = obj_dest.parent() && !parent.exists() {
                                fs::create_dir_all(parent).context(IoSnafu)?
                            }
                            fs::copy(obj_src, obj_dest).context(IoSnafu)?;
                        }
                        ObjectKind::Tree => {
                            // No need to copy anything as long as the path is created
                            // The blobs will be copied anyway
                            if !obj_dest.exists() {
                                fs::create_dir_all(&obj_dest).context(IoSnafu)?
                            }
                        }
                    }

                    assert!(self.objects_mut().insert(package_id, id, object).is_none());
                }
            }

            assert!(self
                .packages_mut()
                .insert(package_id, PackageDesc::new(name, version, blobs, requires))
                .is_none());

            Ok(true)
        }
    }

    pub fn extend<'a, P: AsRef<Path>>(
        &'a mut self,
        packages: impl IntoIterator<Item = (Package, P)> + 'a,
    ) -> impl Iterator<Item = StoreResult<bool>> + 'a {
        packages
            .into_iter()
            .map(|(package, path)| self.insert(package, path))
    }

    /// Links only the package at the specified path
    pub fn link_package(&self, id: &ObjectId, to: &ComponentPaths) -> StoreResult<()> {
        let package = self
            .packages()
            .get(id)
            .ok_or(StoreError::PackageNotFoundById { id: *id });

        todo!()

        // let package = &self.pkgs[index];
        // extra::fs::link_components(
        //     &self.path.join(self.get_package_path(&package, &index)),
        //     package.provides(),
        //     to,
        // )?;
    }

    /// Links all the packages to the specified path.
    pub fn link_packages<'a>(
        &self,
        indices: impl IntoIterator<Item = &'a ObjectId>,
        to: &ComponentPaths,
    ) -> StoreResult<()> {
        for index in indices {
            self.link_package(index, to)?;
        }
        Ok(())
    }

    /// Remove all packages that are currently unused in all generations.
    pub fn remove_unused(&mut self, user_manager: &UserManager) -> StoreResult<HashSet<u64>> {
        let indices = user_manager
            .package_indices()
            .map(|index| *index)
            .collect::<HashSet<ObjectId>>();

        todo!()
    }

    // TODO maybe flush just after every io operation

    /// Flushes all data to the backend
    pub fn flush(self) -> StoreResult<()> {
        self.backend.flush()
    }

    /// The path of the store
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::PACKAGES_DB;
    use crate::{store::backend::LocalBackend, support::*, Store};
    use std::{fs, path::Path};
    use temp_dir::TempDir;

    fn store_create_at_path(path: &Path) -> Store<LocalBackend> {
        let store = Store::<LocalBackend>::init(&path).unwrap();

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

        let res = Store::<LocalBackend>::init(&path);

        assert!(res.is_err());
    }

    #[test]
    fn store_open_ok() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let _store = store_create_at_path(&path);

        let _store = Store::<LocalBackend>::open(path).unwrap();
    }

    #[test]
    fn store_open_err() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");

        let res = Store::<LocalBackend>::open(path);

        assert!(res.is_err());
    }

    #[test]
    fn store_insert() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let package_path = temp_dir.child("package");

        let mut store = store_create_at_path(&path);
        let package = pkg("package", &package_path);
        let package_store_path = package.path_in_store(store.path());

        let _ = store.insert(package, package_path).unwrap();

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
