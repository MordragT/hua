use crate::{extra::path::ComponentPathBuf, UserManager};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
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

    fn get_full_object_path(&self, object_id: &ObjectId) -> Option<PathBuf> {
        if let Some((obj, ext)) = self.objects().get_full(&object_id) {
            let package_path = self.packages().path_in_store(&ext.package_id, &self.path);

            package_path.map(|path| obj.to_path(path))
        } else {
            None
        }
    }

    fn solve_objects(
        id: ObjectId,
        trees: &BTreeMap<ObjectId, Tree>,
        blobs: &BTreeMap<ObjectId, Blob>,
        resolved: &mut HashSet<ObjectId>,
    ) -> bool {
        if let Some(tree) = trees.get(&id) {
            resolved.insert(id);
            for child in &tree.children {
                Self::solve_objects(*child, trees, blobs, resolved);
            }
            true
        } else if let Some(_blob) = blobs.get(&id) {
            resolved.insert(id);
            true
        } else {
            false
        }
    }

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
            blobs,
            trees,
        } = package;

        let mut pkg_blobs = BTreeSet::new();

        if self.packages().contains(&package_id) {
            Ok(false)
        } else {
            let mut resolved = HashSet::new();

            // should be ordered (by BTreeMap and Object::cmp) so that trees with lower depth come first
            for (id, tree) in trees.iter() {
                let dest = tree.to_path(&path_in_store);

                if resolved.contains(id) {
                    continue;
                } else if self.objects().contains(&id) {
                    let original = self
                        .get_full_object_path(&id)
                        .ok_or(StoreError::ObjectNotFoundById { id: *id })?;

                    unix::fs::symlink(original, dest).context(IoSnafu)?;
                    Self::solve_objects(*id, &trees, &blobs, &mut resolved);

                    let ext = unsafe { self.objects_mut().get_ext_mut_unchecked(id) };
                    assert!(ext
                        .links
                        .insert(package_id, Link::new(tree.path.clone(), *id))
                        .is_none());
                } else {
                    fs::create_dir(dest).context(IoSnafu)?;

                    let old = self
                        .objects_mut()
                        .insert(package_id, *id, tree.clone().into());
                    assert!(old.is_none())
                }
            }

            for (id, blob) in blobs {
                pkg_blobs.insert(blob.clone());

                let dest = blob.to_path(&path_in_store);

                if resolved.contains(&id) {
                    continue;
                } else if self.objects().contains(&id) {
                    let original = self
                        .get_full_object_path(&id)
                        .ok_or(StoreError::ObjectNotFoundById { id })?;

                    unix::fs::symlink(original, dest).context(IoSnafu)?;

                    let ext = unsafe { self.objects_mut().get_ext_mut_unchecked(&id) };
                    assert!(ext
                        .links
                        .insert(package_id, Link::new(blob.path, id))
                        .is_none());
                } else {
                    let source = blob.to_path(&path);
                    fs::copy(source, dest).context(IoSnafu)?;

                    let old = self.objects_mut().insert(package_id, id, blob.into());
                    assert!(old.is_none())
                }
            }

            assert!(self
                .packages_mut()
                .insert(
                    package_id,
                    PackageDesc::new(name, version, pkg_blobs, requires)
                )
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
    pub fn link_package(&self, id: &PackageId, to: &ComponentPathBuf) -> StoreResult<()> {
        let root = self
            .packages()
            .path_in_store(id, &self.path)
            .ok_or(StoreError::PackageNotFoundById { id: *id })?;

        to.create_dirs().context(IoSnafu)?;

        self.objects()
            .read_children(id, |object| match object.kind() {
                ObjectKind::Blob => {
                    let absolute = object.to_path(&root);
                    let relative = absolute.strip_prefix(&root).context(StripPrefixSnafu)?;

                    let dest = if relative.starts_with("bin/") {
                        to.binary
                            .join(relative.strip_prefix("bin/").context(StripPrefixSnafu)?)
                    } else if relative.starts_with("lib/") {
                        to.library
                            .join(relative.strip_prefix("lib/").context(StripPrefixSnafu)?)
                    } else if relative.starts_with("cfg/") {
                        to.config
                            .join(relative.strip_prefix("cfg/").context(StripPrefixSnafu)?)
                    } else if relative.starts_with("etc/") {
                        to.config
                            .join(relative.strip_prefix("etc/").context(StripPrefixSnafu)?)
                    } else if relative.starts_with("share/") {
                        to.share
                            .join(relative.strip_prefix("share/").context(StripPrefixSnafu)?)
                    } else {
                        return Err(StoreError::UnsupportedFilePath {
                            path: relative.to_owned(),
                        });
                    };

                    if let Some(parent) = dest.parent() && !parent.exists() {
                        fs::create_dir_all(parent).context(IoSnafu)?;
                    }

                    unix::fs::symlink(absolute, dest).context(IoSnafu)?;

                    Ok(())
                }
                _ => Ok(()),
            })
            .collect::<StoreResult<()>>()?;
        Ok(())
    }

    /// Links all the packages to the specified path.
    pub fn link_packages<'a>(
        &self,
        indices: impl IntoIterator<Item = &'a PackageId>,
        to: &ComponentPathBuf,
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
            .collect::<HashSet<PackageId>>();

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
    use crate::{extra::path::ComponentPathBuf, store::backend::LocalBackend, support::*, Store};
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

    #[test]
    fn store_link_package() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let package_path = temp_dir.child("package");

        let mut store = store_create_at_path(&path);

        let package = pkg("package", &package_path);
        let _package_store_path = package.path_in_store(store.path());
        let package_id = package.id;

        let _ = store.insert(package, package_path).unwrap();

        let global_temp_path = temp_dir.child("global");
        fs::create_dir(&global_temp_path).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp_path);
        global_paths.create_dirs().unwrap();

        store.link_package(&package_id, &global_paths).unwrap();

        // let mut eq_linked = HashSet::new();
        // eq_linked.insert(child_hash);
        // eq_linked.insert(parent_hash);
        // assert_eq!(linked, eq_linked);

        let package_link = global_paths.library.join("package.so");
        assert!(package_link.exists());
        assert!(package_link.is_symlink());
    }
}