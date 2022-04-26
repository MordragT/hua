use super::*;
use crate::{
    dependency::Requirement,
    extra::{self, path::ComponentPathBuf},
    generation::{Generation, GenerationBuilder},
    store::{backend::ReadBackend, id::PackageId, Store},
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::{
    fs,
    path::{Path, PathBuf},
};

// TODO when switching generations also delete the old folders

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenerationManager {
    path: PathBuf,
    counter: usize,
    current: usize,
    generations: HashMap<usize, Generation>,
    global_links: HashSet<PathBuf>,
}

impl GenerationManager {
    pub fn init<P: AsRef<Path>>(path: P) -> GenerationResult<Self> {
        let path = path.as_ref().to_owned();

        let current = 0;
        let mut list = HashMap::new();

        let generation = GenerationBuilder::new(current).under(&path).empty()?;
        list.insert(current, generation);

        Ok(Self {
            path,
            current,
            counter: 0,
            generations: list,
            global_links: HashSet::new(),
        })
    }

    fn unlink_global(&mut self) -> GenerationResult<()> {
        for link in self.global_links.drain() {
            fs::remove_file(link).context(IoSnafu)?;
        }

        Ok(())
    }

    fn link_current_global(&mut self, global_paths: &ComponentPathBuf) -> GenerationResult<()> {
        self.link_global(self.current, global_paths)
    }

    fn link_global(&mut self, id: usize, global_paths: &ComponentPathBuf) -> GenerationResult<()> {
        let gen = self.generations.get(&id).context(NotFoundSnafu { id })?;
        self.global_links = extra::fs::link_component_paths(gen.component_paths(), global_paths)
            .context(IoSnafu)?;
        Ok(())
    }

    pub fn switch_global_links(&mut self, global_paths: &ComponentPathBuf) -> GenerationResult<()> {
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
            match self.generations.remove(&id) {
                Some(gen) => {
                    fs::remove_dir_all(gen.path()).context(IoSnafu)?;
                    Ok(true)
                }
                None => Ok(false),
            }
        }
    }

    pub fn insert_requirement<B: ReadBackend<Source = PathBuf>>(
        &mut self,
        requirement: Requirement,
        store: &Store<PathBuf, B>,
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

            let old = self.generations.insert(self.counter, generation);
            assert!(old.is_none());

            self.current = self.counter;
            Ok(true)
        }
    }

    pub fn remove_requirement<B: ReadBackend<Source = PathBuf>>(
        &mut self,
        requirement: &Requirement,
        store: &Store<PathBuf, B>,
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

            let old = self.generations.insert(self.counter, generation);
            assert!(old.is_none());

            self.current = self.counter;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn packages(&self) -> impl Iterator<Item = &PackageId> {
        self.generations
            .iter()
            .flat_map(|(_id, gen)| gen.packages().iter())
    }

    pub fn current_requirements(&self) -> &HashSet<Requirement> {
        self.get_current().requirements()
    }

    pub fn contains_package(&self, id: &PackageId) -> bool {
        self.packages().find(|idx| *idx == id).is_some()
    }

    pub fn list_current_packages(&self) {
        self.get_current().list_packages();
    }

    pub fn list_generations(&self) {
        for (key, gen) in self.generations.iter() {
            println!("Generation {}\n{}\n", key, gen);
        }
    }

    pub fn switch_to(&mut self, id: usize) -> GenerationResult<()> {
        if self.generations.contains_key(&id) {
            self.current = id;

            Ok(())
        } else {
            Err(GenerationError::NotFound { id })
        }
    }

    fn get_current(&self) -> &Generation {
        unsafe { self.generations.get(&self.current).unwrap_unchecked() }
    }
}
