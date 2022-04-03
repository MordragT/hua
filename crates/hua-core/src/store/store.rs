use crate::{extra::path::ComponentPathBuf, UserManager};
use std::{
    assert_matches::assert_matches,
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
        if let Some(obj) = self.objects().get(&object_id) && let Some(package_id) = self.packages().find_package_id(object_id) {
            let package_path = self.packages().path_in_store(&package_id, &self.path);

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
            let mut object_ids = HashSet::new();

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
                    object_ids.insert(*id);
                } else {
                    fs::create_dir(dest).context(IoSnafu)?;

                    let old = self.objects_mut().insert(*id, tree.clone().into());
                    assert!(old.is_none());
                    object_ids.insert(*id);
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
                    object_ids.insert(id);
                } else {
                    let source = blob.to_path(&path);
                    fs::copy(source, dest).context(IoSnafu)?;

                    let old = self.objects_mut().insert(id, blob.into());
                    assert!(old.is_none());
                    object_ids.insert(id);
                }
            }

            assert!(self
                .packages_mut()
                .insert(
                    package_id,
                    PackageDesc::new(name, version, pkg_blobs, requires),
                    object_ids
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

    /// Links only the package at the specified path.
    /// If the package itself has object linked to other packages,
    /// link to the link.
    pub fn link_package(&self, package_id: &PackageId, to: &ComponentPathBuf) -> StoreResult<()> {
        let root = self
            .packages()
            .path_in_store(package_id, &self.path)
            .ok_or(StoreError::PackageNotFoundById { id: *package_id })?;

        to.create_dirs().context(IoSnafu)?;

        // I checked beforehand if package is in scope
        let object_ids = unsafe { self.packages().get_children(package_id).unwrap_unchecked() };

        self.objects()
            .read_objects(object_ids, |object, _ext| match object.kind() {
                ObjectKind::Blob | ObjectKind::Link => {
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

    fn remove(
        &mut self,
        package_id: &PackageId,
    ) -> Option<(PackageDesc, Vec<(ObjectId, Object, ObjectExt)>)> {
        if let Some((desc, object_ids)) = self.packages_mut().remove(package_id) {
            let objects =
                unsafe { self.objects_mut().remove_objects_unchecked(&object_ids) }.collect();

            Some((desc, objects))
        } else {
            None
        }
    }

    /// Remove all packages that are currently unused in all generations.
    pub fn remove_unused(&mut self, user_manager: &UserManager) -> StoreResult<Vec<PackageId>> {
        let used_packages = user_manager.packages().collect::<HashSet<&PackageId>>();

        let to_remove = self
            .packages()
            .filter(|id, _desc| !used_packages.contains(id))
            .map(|(id, _desc)| *id)
            .collect::<Vec<PackageId>>();

        for package_id in &to_remove {
            let root = unsafe {
                self.packages()
                    .path_in_store(&package_id, &self.path)
                    .unwrap_unchecked()
            };
            let (_desc, objects) = unsafe { self.remove(package_id).unwrap_unchecked() };

            for (object_id, mut object, mut ext) in objects {
                let absolute_src = object.to_path(&root);
                let mut links = ext.links.into_iter();

                if let Some((target, link)) = links.next() {
                    let target_root = unsafe {
                        self.packages()
                            .path_in_store(&target, &self.path)
                            .unwrap_unchecked()
                    };
                    let dest = link.to_path(&target_root);

                    match object.kind() {
                        ObjectKind::Blob => {
                            // Delete link
                            fs::remove_file(&dest).context(IoSnafu)?;
                            fs::copy(absolute_src, dest).context(IoSnafu)?;
                        }
                        ObjectKind::Tree => {
                            fs::remove_dir(&dest).context(IoSnafu)?;
                            fs_extra::dir::move_dir(
                                absolute_src,
                                dest,
                                &fs_extra::dir::CopyOptions::default(),
                            )
                            .context(FsExtraSnafu)?;
                        }
                        ObjectKind::Link => todo!(),
                    }

                    ext.links = links.collect();

                    object.replace_path(link.link);
                    assert!(self
                        .objects_mut()
                        .insert_full(object_id, object, ext)
                        .is_none());
                    assert_matches!(
                        self.packages_mut().insert_child(&target, object_id),
                        Some(true)
                    );
                }
            }

            fs::remove_dir_all(root).context(IoSnafu)?;
        }

        Ok(to_remove)
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
    use crate::{
        extra::path::ComponentPathBuf, store::backend::LocalBackend, support::*, Store, UserManager,
    };
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

    #[test]
    fn store_remove_unused() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let one_path = temp_dir.child("one");
        let two_path = temp_dir.child("two");
        let user_manager_path = temp_dir.child("user");

        let mut store = store_create_at_path(&path);
        let one = pkg("one", &one_path);
        let two = pkg("two", &two_path);

        let two_id = two.id;
        let one_req = to_req(&one);

        store.insert(one, one_path).unwrap();
        store.insert(two, two_path).unwrap();

        let mut user_manager = UserManager::create_at_path(&user_manager_path).unwrap();
        assert!(user_manager
            .insert_requirement(one_req, &mut store)
            .unwrap());

        let removed = store.remove_unused(&mut user_manager).unwrap();

        assert_eq!(removed.len(), 1);
        assert!(removed.contains(&two_id));
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

        let package_link = global_paths.library.join("package.so");
        assert!(package_link.exists());
        assert!(package_link.is_symlink());
    }
}
