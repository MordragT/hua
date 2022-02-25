use crate::{error::*, extra, package::Package, UserManager};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::{self},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    packages: HashMap<u64, Package>,
    hasher: DefaultHasher,
}

impl Store {
    pub fn create_under<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().join("store");

        if !path.exists() {
            fs::create_dir(&path)?;
        }

        Ok(Self {
            path,
            packages: HashMap::new(),
            hasher: DefaultHasher::new(),
        })
    }

    pub fn get(&self, hash: u64) -> Option<&Package> {
        self.packages.get(&hash)
    }

    pub fn insert(&mut self, mut package: Package) -> Result<()> {
        // TODO package is hashed by its name version and contents
        // But the rust default hasher is not stable
        // consider using another one like sha2 or blake3
        package.name.hash(&mut self.hasher);
        package.version.hash(&mut self.hasher);
        extra::hash_path(&package.path, &mut self.hasher)?;
        let hash = self.hasher.finish();

        let name_version_hash = format!("{}-{}-{}", &package.name, &package.version, hash);

        let destination = self.path.join(name_version_hash);
        if !destination.exists() {
            fs::create_dir(&destination)?;

            extra::copy_to(&package.path, &destination)?;
            package.path = destination;

            if let Some(_) = self.packages.insert(hash, package) {
                Err(Error::PackageAlreadyPresent(hash))
            } else {
                Ok(())
            }
        } else {
            Err(Error::PackageAlreadyPresent(hash))
        }
    }

    pub fn remove_unused(&mut self, user_manager: &UserManager) -> Result<()> {
        todo!()
    }
}
