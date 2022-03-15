pub use manager::GenerationManager;
use serde::{Deserialize, Serialize};

use crate::error::*;
use crate::{extra::ComponentPaths, Store};
use std::collections::HashSet;
use std::{
    fmt, fs,
    path::{Path, PathBuf},
};

mod manager;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Generation {
    path: PathBuf,
    packages: HashSet<u64>,
    component_paths: ComponentPaths,
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "Path: {:#?}\nNumber of packages: {}",
            self.path,
            self.packages.len()
        ))
    }
}

impl Generation {
    pub fn create_under<P: AsRef<Path>>(path: P, id: usize) -> Result<Self> {
        let path = path.as_ref().join(id.to_string());
        if path.exists() {
            return Err(Error::GenerationAlreadyPresent(id));
        }
        fs::create_dir(&path)?;
        let component_paths = ComponentPaths::new(
            path.join("bin"),
            path.join("cfg"),
            path.join("lib"),
            path.join("share"),
        );
        component_paths.create_dirs()?;

        Ok(Self {
            path,
            packages: HashSet::new(),
            component_paths,
        })
    }

    pub fn link_package(&mut self, hash: &u64, store: &mut Store) -> Result<()> {
        let mut hashes = store.link_package(hash, &self.component_paths)?;
        let packages = self
            .packages
            .union(&mut hashes)
            .map(|hash| *hash)
            .collect::<HashSet<u64>>();
        self.packages = packages;

        Ok(())
    }

    pub fn link_packages(&mut self, hashes: &HashSet<u64>, store: &mut Store) -> Result<()> {
        let mut hashes = store.link_packages(hashes, &self.component_paths)?;
        let packages = self
            .packages
            .union(&mut hashes)
            .map(|hash| *hash)
            .collect::<HashSet<u64>>();
        self.packages = packages;

        Ok(())
    }

    pub fn list_packages(&self) {
        println!("{:#?}", self.packages);
    }

    pub fn packages(&self) -> &HashSet<u64> {
        &self.packages
    }

    pub fn contains(&self, hash: u64) -> bool {
        self.packages.contains(&hash)
    }

    pub fn binary(&self) -> &Path {
        self.component_paths.binary.as_path()
    }

    pub fn config(&self) -> &Path {
        self.component_paths.config.as_path()
    }

    pub fn library(&self) -> &Path {
        self.component_paths.library.as_path()
    }

    pub fn share(&self) -> &Path {
        self.component_paths.share.as_path()
    }
}
