use super::*;
use crate::{components::ComponentPaths, Store};
use crate::{extra, Generation, GenerationBuilder, Requirement};
use serde::{Deserialize, Serialize};
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
    pub fn create_under<P: AsRef<Path>>(path: P) -> GenerationResult<Self> {
        let path = path.as_ref().join("generations");

        if !path.exists() {
            fs::create_dir(&path).context(IoSnafu)?;
        }

        let current = 0;
        let mut list = HashMap::new();

        let generation = GenerationBuilder::new(current).under(&path).empty()?;
        list.insert(current, generation);

        Ok(Self {
            path,
            current,
            counter: 0,
            list,
            global_links: HashSet::new(),
        })
    }

    fn unlink_global(&mut self) -> GenerationResult<()> {
        for link in self.global_links.drain() {
            fs::remove_file(link).context(IoSnafu)?;
        }

        Ok(())
    }

    fn link_current_global(&mut self, global_paths: &ComponentPaths) -> GenerationResult<()> {
        self.link_global(self.current, global_paths)
    }

    fn link_global(&mut self, id: usize, global_paths: &ComponentPaths) -> GenerationResult<()> {
        let gen = self.list.get(&id).context(NotFoundSnafu { id })?;
        self.global_links = extra::fs::link_component_paths(gen.component_paths(), global_paths)
            .context(IoSnafu)?;
        Ok(())
    }

    pub fn switch_global_links(&mut self, global_paths: &ComponentPaths) -> GenerationResult<()> {
        self.unlink_global()?;
        self.link_current_global(global_paths)
    }

    pub fn global_links(&self) -> &HashSet<PathBuf> {
        &self.global_links
    }

    pub fn remove_generation(&mut self, id: usize) -> GenerationResult<bool> {
        if id == self.current {
            Err(GenerationError::InUse { id })
        } else {
            match self.list.remove(&id) {
                Some(gen) => {
                    fs::remove_dir_all(gen.path()).context(IoSnafu)?;
                    Ok(true)
                }
                None => Ok(false),
            }
        }
    }

    pub fn insert_requirement(
        &mut self,
        requirement: Requirement,
        store: &Store,
    ) -> GenerationResult<bool> {
        if self.get_current().contains_requirement(&requirement) {
            Ok(false)
        } else {
            self.counter += 1;

            let mut requirements = self.get_current().requirements().clone();
            requirements.insert(requirement);

            let generation = GenerationBuilder::new(self.counter)
                .under(&self.path)
                .requires(requirements)
                .resolve(store)?
                .build(store)?;

            let old = self.list.insert(self.counter, generation);
            assert!(old.is_none());

            self.current = self.counter;
            Ok(true)
        }
    }

    pub fn remove_requirement(
        &mut self,
        requirement: &Requirement,
        store: &Store,
    ) -> GenerationResult<bool> {
        if self.get_current().contains_requirement(requirement) {
            self.counter += 1;

            let mut requirements = self.get_current().requirements().clone();
            requirements.remove(requirement);

            let generation = GenerationBuilder::new(self.counter)
                .under(&self.path)
                .requires(requirements)
                .resolve(store)?
                .build(store)?;

            let old = self.list.insert(self.counter, generation);
            assert!(old.is_none());

            self.current = self.counter;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn packages(&self) -> impl Iterator<Item = &usize> {
        self.list
            .iter()
            .flat_map(|(_id, gen)| gen.packages().iter())
    }

    pub fn contains_package(&self, index: usize) -> bool {
        self.packages().find(|idx| **idx == index).is_some()
    }

    pub fn list_current_packages(&self) {
        self.get_current().list_packages();
    }

    pub fn list_generations(&self) {
        for (key, gen) in self.list.iter() {
            println!("Generation {}\n{}\n", key, gen);
        }
    }

    pub fn switch_to(&mut self, id: usize) -> GenerationResult<()> {
        if self.list.contains_key(&id) {
            self.current = id;

            Ok(())
        } else {
            Err(GenerationError::NotFound { id })
        }
    }

    fn get_current(&self) -> &Generation {
        unsafe { self.list.get(&self.current).unwrap_unchecked() }
    }
}
