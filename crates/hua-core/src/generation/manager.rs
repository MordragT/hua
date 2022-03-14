use serde::{Deserialize, Serialize};

use crate::{error::*, extra, Generation};
use crate::{ComponentPaths, Store};
use std::collections::HashMap;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenerationManager {
    path: PathBuf,
    current: usize,
    list: HashMap<usize, Generation>,
}

impl GenerationManager {
    pub fn create_under<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().join("generations");

        if !path.exists() {
            fs::create_dir(&path)?;
        }

        let current = 0;
        let mut list = HashMap::new();
        list.insert(current, Generation::create_under(&path, current)?);

        Ok(Self {
            path,
            current,
            list,
        })
    }

    pub fn link_current_global(&self, global_paths: ComponentPaths) -> Result<()> {
        self.link_global(self.current, global_paths)
    }

    pub fn link_global(&self, id: usize, global_paths: ComponentPaths) -> Result<()> {
        let gen = self.list.get(&id).ok_or(Error::GenerationNotFound(id))?;
        extra::link_to(gen.binary(), global_paths.binary)?;
        extra::link_to(gen.config(), global_paths.config)?;
        extra::link_to(gen.library(), global_paths.library)?;
        extra::link_to(gen.share(), global_paths.share)?;
        Ok(())
    }

    pub fn remove_generation(&mut self, id: usize) -> Result<()> {
        match self.list.remove(&id) {
            Some(gen) => Ok(fs::remove_dir_all(gen.path)?),
            None => Err(Error::GenerationNotFound(id)),
        }
    }

    pub fn insert_package(&mut self, hash: &u64, store: &mut Store) -> Result<()> {
        let id = self.list.len();
        let mut gen = Generation::create_under(&self.path, id)?;

        gen.link_packages(self.get_current().packages(), store)?;
        gen.link_package(hash, store)?;

        let old = self.list.insert(id, gen);
        assert!(old.is_none());

        self.current = id;
        Ok(())
    }

    pub fn packages(&self) -> impl Iterator<Item = &u64> {
        self.list.iter().flat_map(|(_id, gen)| &gen.packages)
    }

    pub fn list_current_packages(&self) {
        self.get_current().list_packages();
    }

    pub fn switch_to(&mut self, id: usize) -> Result<()> {
        if self.list.contains_key(&id) {
            self.current = id;

            Ok(())
        } else {
            Err(Error::GenerationNotFound(id))
        }
    }

    fn get_current(&self) -> &Generation {
        self.list
            .get(&self.current)
            .expect("The current generation should always be present in the manager")
    }

    fn get_current_mut(&mut self) -> &mut Generation {
        self.list
            .get_mut(&self.current)
            .expect("The current generation should always be present in the manager")
    }
}
