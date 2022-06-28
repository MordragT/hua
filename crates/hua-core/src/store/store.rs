use super::{
    backend::{LocalBackend, MemoryBackend, ReadBackend, RemoteBackend, WriteBackend},
    derivation::{Derivation, Derivations},
    object::{Blob, Objects},
    package::{PackageDesc, Packages},
    *,
};
use crate::{
    dependency::Requirement,
    extra::{
        hash::{Blake3, PackageHash},
        path::ComponentPathBuf,
        style::ProgressBar,
    },
    user::UserManager,
    GID, UID,
};
use log::{info, warn};
use rs_merkle::Hasher;
use std::{
    collections::HashSet,
    fs::{self},
    ops::Deref,
    os::unix::{self},
    path::{Path, PathBuf},
};
use url::Url;

/// The filename of the packages database of the store
const PACKAGES_DB: &str = "packages.db";
pub const STORE_PATH: &str = "/hua/store/";

pub type LocalStore = Store<PathBuf, LocalBackend>;
pub type RemoteStore = Store<Url, RemoteBackend>;
pub type MemoryStore = Store<(), MemoryBackend>;

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

impl Store<(), MemoryBackend> {
    pub fn init() -> StoreResult<Self> {
        let backend = MemoryBackend::init(Box::new(()))?;
        Ok(Self {
            backend,
            source: (),
        })
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

        // I checked beforehand if package is in scope
        let object_ids = unsafe { self.packages().get_children(package_id).unwrap_unchecked() };

        for (_id, object) in self.objects().get_multiple(object_ids) {
            match object.kind() {
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
                        warn!("Unsupported file path {relative:?}");
                        continue;
                    };

                    if let Some(parent) = dest.parent() && !parent.exists() {
                        fs::create_dir_all(parent).context(IoSnafu)?;
                    }

                    unix::fs::symlink(absolute, dest).context(IoSnafu)?;
                }
                _ => (),
            }
        }

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
        unix::fs::chown(&path, UID, GID).context(IoSnafu)?;

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

    pub fn derivations(&self) -> &Derivations {
        self.backend.derivations()
    }

    pub fn matches<'a>(
        &'a self,
        requirement: &'a Requirement,
    ) -> impl Iterator<
        Item = (
            &'a PackageId,
            &'a PackageDesc,
            impl Iterator<Item = &'a Blob>,
            &'a Derivation,
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
                    let drv_id = self.derivations().get_drv_of_pkg(id).unwrap();
                    let drv = self.derivations().get(&drv_id).unwrap();

                    Some((id, desc, self.objects().get_blobs(objects), drv))
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

    pub fn derivations_mut(&mut self) -> &mut Derivations {
        self.backend.derivations_mut()
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
    pub fn insert(&mut self, source: PackageSource) -> StoreResult<Option<PathBuf>> {
        // let (verified, trees, blobs) = package.verify(&path).context(VerifyIoSnafu)?;

        // if !verified {
        //     return Err(StoreError::PackageNotVerified { package });
        // }

        // info!("Verified {package}");

        let PackageSource { desc, drv, path } = source;

        let PackageHash {
            root: package_id,
            trees,
            blobs,
        } = PackageHash::from_path(&path, &desc.name).context(IoSnafu)?;

        let path_in_store = desc.path_in_store(&self.source, package_id);
        fs::create_dir(&path_in_store).context(IoSnafu)?;
        unix::fs::chown(&path_in_store, UID, GID).context(IoSnafu)?;

        info!("Created {path_in_store:?}");

        // let Package {
        //     id: package_id,
        //     desc,
        // } = package;

        let PackageDesc {
            name,
            desc,
            version,
            licenses,
        } = desc;

        if self.packages().contains(&package_id) {
            Ok(None)
        } else {
            let mut object_ids: HashSet<ObjectId> = HashSet::new();

            // should be ordered (by BTreeMap and Object::cmp) so that trees with lower depth come first
            for (tree, id) in trees {
                let dest = tree.to_path(&path_in_store);
                fs::create_dir(&dest).context(CreateTreeSnafu { path: dest.clone() })?;
                unix::fs::chown(&dest, UID, GID).context(IoSnafu)?;
                object_ids.insert(id);

                if !self.objects().contains(&id) {
                    self.objects_mut().insert(id, tree.into());
                }
            }

            info!("Package directories created");

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
                        dest: dest.clone(),
                    })?;
                    unix::fs::chown(&dest, UID, GID).context(IoSnafu)?;

                    let old = self.objects_mut().insert(id, blob.into());
                    assert!(old.is_none());
                    object_ids.insert(id);
                }
            }

            info!("Blobs copied or linked");

            assert!(self
                .packages_mut()
                .insert(
                    package_id,
                    PackageDesc::new(name, desc, version, licenses),
                    object_ids
                )
                .is_none());

            let drv_id = package_id.into();
            let mut provides = HashSet::new();
            provides.insert(package_id);
            assert!(self
                .derivations_mut()
                .insert(drv_id, drv, provides)
                .is_none());

            Ok(Some(path_in_store))
        }
    }

    pub fn extend<'a>(
        &'a mut self,
        packages: impl IntoIterator<Item = PackageSource> + 'a,
    ) -> impl Iterator<Item = StoreResult<Option<PathBuf>>> + 'a {
        packages.into_iter().map(|src| self.insert(src))
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
                let (_desc, objects) =
                    unsafe { self.packages_mut().remove(package_id).unwrap_unchecked() };
                assert!(self
                    .objects_mut()
                    .remove_objects(objects.iter())
                    .collect::<Option<Vec<_>>>()
                    .is_some());

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
                let (_desc, objects) =
                    unsafe { self.packages_mut().remove(package_id).unwrap_unchecked() };
                assert!(self
                    .objects_mut()
                    .remove_objects(objects.iter())
                    .collect::<Option<Vec<_>>>()
                    .is_some());
            }
        }

        Ok(to_remove)
    }

    /// Flushes all data to the backend
    pub fn flush(self) -> StoreResult<()> {
        self.backend.flush()?;

        // let path = self.source.join(PACKAGES_DB);
        // let mut perm = fs::metadata(&path).context(IoSnafu)?.permissions();
        // perm.set_mode(0o644);
        // fs::set_permissions(path, perm).context(IoSnafu)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalStore, PACKAGES_DB};
    use crate::{
        extra::{hash, path::ComponentPathBuf},
        support::*,
        user::UserManager,
    };
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
        let package_id = hash::root_hash(&package).unwrap();
        let package_store_path = package.path_in_store(store.path(), package_id);

        let _ = store.insert(package).unwrap();

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

        let two_id = hash::root_hash(&two).unwrap();
        let one_req = req("one", ">0.0.0");

        store.insert(one).unwrap();
        store.insert(two).unwrap();

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
        let package_id = hash::root_hash(&package).unwrap();
        let _package_store_path = package.path_in_store(store.path(), package_id);

        let _ = store.insert(package).unwrap();

        let global_temp_path = temp_dir.child("global");
        fs::create_dir(&global_temp_path).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp_path);
        global_paths.create_dirs(true).unwrap();

        store.link_package(&package_id, &global_paths).unwrap();

        let package_link = global_paths.library.join("package.so");
        assert!(package_link.exists());
        assert!(package_link.is_symlink());
    }
}
