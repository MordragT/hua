pub use manager::GenerationManager;
use serde::{Deserialize, Serialize};

use crate::{components::ComponentPaths, Store};
use crate::{error::*, Requirement};
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
    //requires: HashSet<Box<dyn Dependency>>,
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
            //requires: HashSet::new(),
        })
    }

    pub fn resolve_dependencies(
        &self,
        dependencies: &HashSet<Requirement>,
        store: &mut Store,
    ) -> Result<HashSet<u64>> {
        todo!()
    }

    /// Checks if the given dependency conflicts with any other already installed
    /// and returns a list of dependencies currently not installed.
    pub fn resolve_dependency(
        &self,
        requirement: &Requirement,
        store: &mut Store,
    ) -> Result<HashSet<u64>> {
        // for key in &self.packages {
        //     store.read_packages(|map| {
        //         let package = map
        //             .get(&key)
        //             .expect("Every package in the generation must be in the store.");
        //         if !package.provides.is_disjoint(requirement.components()) {
        //             Err(Error::DependencyProvideCollision(requirement.clone()))
        //         } else {
        //             Ok(())
        //         }
        //     })??;
        // }

        todo!()
    }

    pub fn link_package(&mut self, index: usize, store: &mut Store) -> Result<()> {
        // let mut hashes = store.link_package(hash, &self.component_paths)?;
        // let packages = self
        //     .packages
        //     .union(&mut hashes)
        //     .map(|hash| *hash)
        //     .collect::<HashSet<u64>>();
        // self.packages = packages;

        Ok(())
    }

    pub fn link_packages(&mut self, hashes: &HashSet<u64>, store: &mut Store) -> Result<()> {
        // let mut hashes = store.link_packages(hashes, &self.component_paths)?;
        // let packages = self
        //     .packages
        //     .union(&mut hashes)
        //     .map(|hash| *hash)
        //     .collect::<HashSet<u64>>();
        // self.packages = packages;

        Ok(())
    }

    pub fn list_packages(&self) {
        println!("{:#?}", self.packages);
    }

    pub fn packages(&self) -> &HashSet<u64> {
        &self.packages
    }

    pub fn contains(&self, index: usize) -> bool {
        // self.packages.contains(index)
        todo!()
    }

    pub fn component_paths(&self) -> &ComponentPaths {
        &self.component_paths
    }

    // pub fn binary(&self) -> &Path {
    //     self.component_paths.binary.as_path()
    // }

    // pub fn config(&self) -> &Path {
    //     self.component_paths.config.as_path()
    // }

    // pub fn library(&self) -> &Path {
    //     self.component_paths.library.as_path()
    // }

    // pub fn share(&self) -> &Path {
    //     self.component_paths.share.as_path()
    // }
}
