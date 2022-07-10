use super::*;
use crate::{
    dependency::Requirement,
    extra::{self, path::ComponentPathBuf},
    generation::{Generation, GenerationBuilder},
    store::{backend::ReadBackend, id::PackageId, Store},
};
use console::style;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt, fs,
    path::{Path, PathBuf},
};

// TODO when switching generations also delete the old folders

/// A Manager for different [generations](Generation).
/// Usually managed itself by a [crate::user::User].
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenerationManager {
    path: PathBuf,
    counter: usize,
    current: usize,
    generations: HashMap<usize, Generation>,
    global_links: HashSet<PathBuf>,
}

impl GenerationManager {
    /// Initialises a new [GenerationManager] under the specified [Path].
    pub fn init<P: AsRef<Path>>(path: P) -> GenerationResult<Self> {
        let path = path.as_ref().to_owned();
        fs::create_dir(&path).context(IoSnafu)?;

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

    /// If the current [Generation] does not contain the [Requirement],
    /// create a copy of the current [Generation] and add the [Requirement],
    /// then return true.
    /// If the [Requirement] is already contained by the [Generation], return false.
    pub fn insert_requirement<B: ReadBackend<Source = PathBuf>>(
        &mut self,
        requirement: Requirement,
        store: &Store<PathBuf, B>,
    ) -> GenerationResult<bool> {
        if self.current().contains_requirement(&requirement) {
            Ok(false)
        } else {
            self.counter += 1;

            let mut requirements = self.current().requirements().clone();
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

    /// If the current [Generation] contains the specified [Requirement],
    /// creates a copy of the current [Generation] with the specified [Requirement] removed
    /// and make the copy the current [Generation], then return true.
    /// If the current [Generation] does not contain the specified [Requirement] just return false.
    pub fn remove_requirement<B: ReadBackend<Source = PathBuf>>(
        &mut self,
        requirement: &Requirement,
        store: &Store<PathBuf, B>,
    ) -> GenerationResult<bool> {
        if self.current().contains_requirement(requirement) {
            self.counter += 1;

            let mut requirements = self.current().requirements().clone();
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

    /// Removes a [Generation].
    /// Returns an [GenerationError] if the [Generation] is still in use,
    /// and returns true if the [Generation] was deleted and false if
    /// it did not exist in the first place.
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

    /// Switches the current [Generation] to the specified id.
    /// Returns an [GenerationError] if the id was not found.
    pub fn switch_to(
        &mut self,
        id: usize,
        global_paths: &ComponentPathBuf,
    ) -> GenerationResult<()> {
        if self.generations.contains_key(&id) {
            self.current = id;

            self.switch_global_links(global_paths)?;

            Ok(())
        } else {
            Err(GenerationError::NotFound { id })
        }
    }

    /// Switches the global links to the newest [Generation].
    pub fn switch_global_links(&mut self, global_paths: &ComponentPathBuf) -> GenerationResult<()> {
        self.unlink_global()?;
        self.link_current_global(global_paths)
    }

    /// Returns a collection of every [PathBuf] of a global link.
    pub fn global_links(&self) -> &HashSet<PathBuf> {
        &self.global_links
    }

    /// Returns an [Iterator] of every [PackageId] respective [Package](crate::store::Package) inside any
    /// of the [generations](Generation) managed.
    pub fn packages(&self) -> impl Iterator<Item = &PackageId> {
        self.generations
            .iter()
            .flat_map(|(_id, gen)| gen.packages().iter())
    }

    /// Checks if all [generations](Generation) managed, contain the specified id.
    pub fn contains_package(&self, id: &PackageId) -> bool {
        self.packages().find(|idx| *idx == id).is_some()
    }

    /// Returns the current [Generation] in use by the [GenerationManager].
    pub fn current(&self) -> &Generation {
        unsafe { self.generations.get(&self.current).unwrap_unchecked() }
    }

    /// Get all [generations](Generation).
    pub fn generations(&self) -> &HashMap<usize, Generation> {
        &self.generations
    }

    fn unlink_global(&mut self) -> GenerationResult<()> {
        for link in self.global_links.drain() {
            fs::remove_file(link).context(IoSnafu)?;
        }

        Ok(())
    }

    fn link_global(&mut self, id: usize, global_paths: &ComponentPathBuf) -> GenerationResult<()> {
        let gen = self.generations.get(&id).context(NotFoundSnafu { id })?;
        self.global_links = extra::fs::link_component_paths(gen.component_paths(), global_paths)
            .context(IoSnafu)?;
        Ok(())
    }

    fn link_current_global(&mut self, global_paths: &ComponentPathBuf) -> GenerationResult<()> {
        self.link_global(self.current, global_paths)
    }
}

impl fmt::Display for GenerationManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "GenerationManager at {:#?}\nCurrent generation {}\n\n",
            &self.path, &self.current
        )?;
        for (id, gen) in &self.generations {
            write!(
                f,
                "{} {}:\n{gen}\n",
                style("Generation").green(),
                style(id).blue()
            )?;
        }
        Ok(())
    }
}
