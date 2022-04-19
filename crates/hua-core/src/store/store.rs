use url::Url;

use crate::{
    dependency::Requirement,
    extra::{path::ComponentPathBuf, style::ProgressBar},
    user::UserManager,
};
use std::{
    collections::HashSet,
    fs::{self},
    os::unix,
    path::{Path, PathBuf},
};

use super::{
    backend::{LocalBackend, ReadBackend, RemoteBackend, WriteBackend},
    object::{Blob, Objects},
    package::{PackageDesc, Packages},
    *,
};

/// The filename of the packages database of the store
const PACKAGES_DB: &str = "packages.db";
pub const STORE_PATH: &str = "/hua/store/";

pub type LocalStore = Store<PathBuf, LocalBackend>;
pub type RemoteStore = Store<Url, RemoteBackend>;

/// A Store that contains all the packages installed by any user
/// Content Addressable Store
#[derive(Debug)]
pub struct Store<S, B, const BAR: bool = true> {
    source: S,
    backend: B,
}

impl<B> Store<Url, B> {
    pub fn url(&self) -> &Url {
        &self.source
    }
}

impl<B: ReadBackend<Source = Url>> Store<Url, B> {
    pub fn open(url: Url) -> StoreResult<Self> {
        let packages_db_url = url.join(PACKAGES_DB).context(UrlParseSnafu)?;
        let backend = B::open(packages_db_url)?;

        Ok(Self {
            source: url,
            backend,
        })
    }
}

impl<B> Store<PathBuf, B> {
    pub fn path(&self) -> &Path {
        &self.source
    }
}

impl<B: ReadBackend<Source = PathBuf>> Store<PathBuf, B> {
    /// Opens a store under the specified path.
    /// Returns an error if the path does not exists or
    /// does not contain the necessary files
    pub fn open<P: AsRef<Path>>(path: P) -> StoreResult<Self> {
        let path = path.as_ref().to_owned();

        if !path.exists() {
            return Err(StoreError::NotExisting { path });
        }

        let backend = B::open(path.join(PACKAGES_DB))?;

        Ok(Self {
            source: path,
            backend,
        })
    }

    /// Links only the package at the specified path.
    /// If the package itself has object linked to other packages,
    /// link to the link.
    pub fn link_package(&self, package_id: &PackageId, to: &ComponentPathBuf) -> StoreResult<()> {
        let root = self
            .packages()
            .path_in_store(package_id, &self.source)
            .ok_or(StoreError::PackageNotFoundById { id: *package_id })?;

        to.create_dirs().context(IoSnafu)?;

        // I checked beforehand if package is in scope
        let object_ids = unsafe { self.packages().get_children(package_id).unwrap_unchecked() };

        self.objects()
            .read_objects(object_ids, |_id, object| match object.kind() {
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
}

impl<B: WriteBackend<Source = PathBuf>> Store<PathBuf, B> {
    /// Creates a new store directory under the given path.
    /// Will return an Error if the directory already exists
    pub fn init<P: AsRef<Path>>(path: P) -> StoreResult<Self> {
        let path = path.as_ref().to_owned();
        fs::create_dir(&path).context(IoSnafu)?;

        let backend = B::init(path.join(PACKAGES_DB))?;

        Ok(Self {
            source: path,
            backend,
        })
    }
}

impl<S, B: ReadBackend, const BAR: bool> Store<S, B, BAR> {
    pub fn packages(&self) -> &Packages {
        self.backend.packages()
    }

    pub fn objects(&self) -> &Objects {
        self.backend.objects()
    }

    pub fn matches<'a>(
        &'a self,
        requirement: &'a Requirement,
    ) -> impl Iterator<
        Item = (
            &'a PackageId,
            &'a PackageDesc,
            impl Iterator<Item = &'a Blob>,
        ),
    > + '_ {
        self.packages()
            .filter(|_id, desc, _objects| {
                requirement.name() == &desc.name && requirement.version_req().matches(&desc.version)
            })
            .filter_map(|(id, desc, objects)| {
                // TODO find a way to not get blobs two times

                let blobs = self.objects().get_blobs_cloned(objects).collect();
                if requirement.blobs().is_subset(&blobs) {
                    Some((id, desc, self.objects().get_blobs(objects)))
                } else {
                    None
                }
            })
    }

    pub fn is_matching(&self, package_id: &PackageId, requirement: &Requirement) -> bool {
        if let Some((desc, objects)) = self.packages().get_full(package_id) {
            let blobs = self.objects().get_blobs_cloned(objects).collect();
            requirement.blobs().is_subset(&blobs)
                && requirement.name() == &desc.name
                && requirement.version_req().matches(&desc.version)
        } else {
            false
        }
    }

    pub fn get_blobs_of_package<'a>(
        &'a self,
        package_id: &PackageId,
    ) -> Option<impl Iterator<Item = &'a Blob>> {
        if let Some(objects) = self.packages().get_children(package_id) {
            Some(self.objects().get_blobs(objects))
        } else {
            None
        }
    }

    pub fn get_blobs_cloned_of_package(
        &self,
        package_id: &PackageId,
    ) -> Option<impl Iterator<Item = Blob> + '_> {
        if let Some(objects) = self.packages().get_children(package_id) {
            Some(self.objects().get_blobs_cloned(objects))
        } else {
            None
        }
    }
}

impl<S, B: WriteBackend, const BAR: bool> Store<S, B, BAR> {
    pub fn objects_mut(&mut self) -> &mut Objects {
        self.backend.objects_mut()
    }

    pub fn packages_mut(&mut self) -> &mut Packages {
        self.backend.packages_mut()
    }
}

impl<B: WriteBackend<Source = PathBuf> + ReadBackend<Source = PathBuf>, const BAR: bool>
    Store<PathBuf, B, BAR>
{
    fn get_full_object_path(&self, object_id: &ObjectId) -> Option<PathBuf> {
        if let Some(obj) = self.objects().get(&object_id) && let Some(package_id) = self.packages().find_package_id(object_id) {
            let package_path = self.packages().path_in_store(&package_id, &self.source);

            package_path.map(|path| obj.to_path(path))
        } else {
            None
        }
    }

    /// Inserts a package into the store and returns true if the package was not present and was inserted
    /// and false if it was already present.
    pub fn insert<P: AsRef<Path>>(&mut self, package: Package, path: P) -> StoreResult<bool> {
        let (verified, trees, blobs) = package.verify(&path).context(IoSnafu)?;

        if !verified {
            return Err(StoreError::PackageNotVerified { package });
        }

        let path_in_store = package.path_in_store(&self.source);
        fs::create_dir(&path_in_store).context(IoSnafu)?;

        let Package {
            id: package_id,
            desc,
        } = package;

        let PackageDesc {
            name,
            desc,
            version,
            licenses,
            requires,
        } = desc;

        if self.packages().contains(&package_id) {
            Ok(false)
        } else {
            let mut object_ids: HashSet<ObjectId> = HashSet::new();

            // should be ordered (by BTreeMap and Object::cmp) so that trees with lower depth come first
            for (tree, id) in trees {
                let dest = tree.to_path(&path_in_store);
                fs::create_dir(&dest).context(CreateTreeSnafu { path: dest })?;
                object_ids.insert(id);

                if !self.objects().contains(&id) {
                    self.objects_mut().insert(id, tree.into());
                }
            }

            for (blob, id) in blobs {
                let dest = blob.to_path(&path_in_store);

                if self.objects().contains(&id) {
                    // Object is saved under other package
                    if let Some(original) = self.get_full_object_path(&id) {
                        fs::hard_link(&original, &dest).context(LinkObjectsSnafu {
                            kind: ObjectKind::Blob,
                            original,
                            link: dest,
                        })?;
                    } else {
                        // Object is saved under current package
                        let object = unsafe { self.objects().get_unchecked(&id) };
                        let original = object.to_path(&path_in_store);
                        if original.exists() {
                            fs::hard_link(&original, &dest).context(LinkObjectsSnafu {
                                kind: ObjectKind::Blob,
                                original,
                                link: dest,
                            })?;
                        } else {
                            return Err(StoreError::ObjectNotRetrievable {
                                object: object.clone(),
                            });
                        }
                    }
                    object_ids.insert(id);
                } else {
                    let source = blob.to_path(&path);
                    fs::copy(&source, &dest).context(CopyObjectSnafu {
                        kind: ObjectKind::Blob,
                        src: source,
                        dest,
                    })?;

                    let old = self.objects_mut().insert(id, blob.into());
                    assert!(old.is_none());
                    object_ids.insert(id);
                }
            }

            assert!(self
                .packages_mut()
                .insert(
                    package_id,
                    PackageDesc::new(name, desc, version, licenses, requires),
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

    /// Remove all packages that are currently unused in all generations.
    pub fn remove_unused(&mut self, user_manager: &UserManager) -> StoreResult<Vec<PackageId>> {
        let used_packages = user_manager.packages().collect::<HashSet<&PackageId>>();

        let to_remove = self
            .packages()
            .filter(|id, _desc, _objects| !used_packages.contains(id))
            .map(|(id, _desc, _objects)| *id)
            .collect::<Vec<PackageId>>();

        if BAR {
            let mut bar = ProgressBar::new(to_remove.len() as u64);

            for package_id in &to_remove {
                let root = unsafe {
                    self.packages()
                        .path_in_store(&package_id, &self.source)
                        .unwrap_unchecked()
                };

                fs::remove_dir_all(root).context(IoSnafu)?;
                bar.inc(1);
            }

            bar.finish("Unused packages removed");
        } else {
            for package_id in &to_remove {
                let root = unsafe {
                    self.packages()
                        .path_in_store(&package_id, &self.source)
                        .unwrap_unchecked()
                };

                fs::remove_dir_all(root).context(IoSnafu)?;
            }
        }

        Ok(to_remove)
    }

    /// Flushes all data to the backend
    pub fn flush(self) -> StoreResult<()> {
        self.backend.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalStore, PACKAGES_DB};
    use crate::{extra::path::ComponentPathBuf, support::*, user::UserManager};
    use std::{fs, path::Path};
    use temp_dir::TempDir;

    fn store_create_at_path(path: &Path) -> LocalStore {
        let store = LocalStore::init(&path).unwrap();

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

        let res = LocalStore::init(&path);

        assert!(res.is_err());
    }

    #[test]
    fn store_open_ok() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");
        let _store = store_create_at_path(&path);

        let _store = LocalStore::open(path).unwrap();
    }

    #[test]
    fn store_open_err() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("store");

        let res = LocalStore::open(path);

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
        let one_req = req("one", ">0.0.0");

        store.insert(one, one_path).unwrap();
        store.insert(two, two_path).unwrap();

        let mut user_manager = UserManager::init(&user_manager_path).unwrap();
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
        let mut user_manager = UserManager::init(&user_manager_path).unwrap();

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
