use serde::{Deserialize, Serialize};

use crate::{components::ComponentPaths, Store};
use crate::{error::*, extra, Generation};
use std::collections::{HashMap, HashSet};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenerationManager {
    path: PathBuf,
    counter: usize,
    current: usize,
    list: HashMap<usize, Generation>,
    global_links: HashSet<PathBuf>,
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
            counter: 0,
            list,
            global_links: HashSet::new(),
        })
    }

    fn unlink_global(&mut self) -> Result<()> {
        for link in self.global_links.drain() {
            fs::remove_file(link)?;
        }

        Ok(())
    }

    fn link_current_global(&mut self, global_paths: &ComponentPaths) -> Result<()> {
        self.link_global(self.current, global_paths)
    }

    fn link_global(&mut self, id: usize, global_paths: &ComponentPaths) -> Result<()> {
        let gen = self.list.get(&id).ok_or(Error::GenerationNotFound(id))?;
        self.global_links = extra::fs::link_component_paths(gen.component_paths(), global_paths)?;
        Ok(())
    }

    pub fn switch_global_links(&mut self, global_paths: &ComponentPaths) -> Result<()> {
        self.unlink_global()?;
        self.link_current_global(global_paths)
    }

    pub fn global_links(&self) -> &HashSet<PathBuf> {
        &self.global_links
    }

    pub fn remove_generation(&mut self, id: usize) -> Result<bool> {
        if id == self.current {
            Err(Error::GenerationIsInUse)
        } else {
            match self.list.remove(&id) {
                Some(gen) => {
                    fs::remove_dir_all(gen.path)?;
                    Ok(true)
                }
                None => Ok(false),
            }
        }
    }

    pub fn insert_package(&mut self, index: usize, store: &mut Store) -> Result<bool> {
        if self.get_current().contains(index) {
            Ok(false)
        } else {
            self.counter += 1;
            let mut gen = Generation::create_under(&self.path, self.counter)?;

            gen.link_packages(self.get_current().packages(), store)?;
            gen.link_package(index, store)?;

            let old = self.list.insert(self.counter, gen);
            assert!(old.is_none());

            self.current = self.counter;
            Ok(true)
        }
    }

    pub fn remove_package(&mut self, index: usize, store: &mut Store) -> Result<bool> {
        // if self.get_current().contains(hash) {
        //     self.counter += 1;
        //     let mut gen = Generation::create_under(&self.path, self.counter)?;

        //     let mut to_remove = store.get_children(hash)?;
        //     to_remove.insert(*hash);

        //     let packages = self
        //         .get_current()
        //         .packages()
        //         .difference(&to_remove)
        //         .map(|hash| *hash)
        //         .collect::<HashSet<u64>>();

        //     gen.link_packages(&packages, store)?;

        //     let old = self.list.insert(self.counter, gen);
        //     assert!(old.is_none());

        //     self.current = self.counter;
        //     Ok(true)
        // } else {
        //     Ok(false)
        // }
        todo!()
    }

    pub fn packages(&self) -> impl Iterator<Item = &u64> {
        self.list.iter().flat_map(|(_id, gen)| &gen.packages)
    }

    pub fn list_current_packages(&self) {
        self.get_current().list_packages();
    }

    pub fn list_generations(&self) {
        for (key, gen) in self.list.iter() {
            println!("Generation {}\n{}\n", key, gen);
        }
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

    #[allow(dead_code)]
    fn get_current_mut(&mut self) -> &mut Generation {
        self.list
            .get_mut(&self.current)
            .expect("The current generation should always be present in the manager")
    }
}
