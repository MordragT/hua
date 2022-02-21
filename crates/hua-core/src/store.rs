use crate::{error::*, package::Package};
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
        let path = path.as_ref().join("/store");
        fs::create_dir(&path)?;
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
        hash_path(&package.path, &mut self.hasher)?;
        let hash = self.hasher.finish();

        let name_version_hash = format!("{}-{}-{}", &package.name, &package.version, hash);

        let destination = self.path.join(name_version_hash);
        fs::create_dir(&destination)?;

        let _number_of_files = fs::copy(&package.path, &destination)?;
        package.path = destination;

        if let Some(p) = self.packages.insert(hash, package) {
            Err(Error::PackageAlreadyPresent(p))
        } else {
            Ok(())
        }
    }
}

fn hash_path<P: AsRef<Path>, H: Hasher>(path: P, state: &mut H) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        path.read_dir()?
            .map(|res| match res {
                Ok(entry) => hash_path(entry.path(), state),
                Err(e) => Err(e.into()),
            })
            .collect::<Result<()>>()
    } else {
        fs::read(path)?.hash(state);
        Ok(())
    }
}
