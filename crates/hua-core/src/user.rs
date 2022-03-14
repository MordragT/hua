use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::generation::GenerationManager;
use crate::{error::*, ComponentPaths, Store};

#[derive(Debug)]
pub struct User {
    name: String,
    path: PathBuf,
    generation_manager: GenerationManager,
}

impl User {
    pub fn create_under<P: AsRef<Path>>(path: P) -> Result<Self> {
        let name = users::get_current_username()
            .ok_or(Error::UsernameNotFound)?
            .into_string()?;

        let path = path.as_ref().join(&name);
        if !path.exists() {
            fs::create_dir(&path)?;
        }

        let generation_manager = GenerationManager::create_under(&path)?;

        Ok(Self {
            name,
            path,
            generation_manager,
        })
    }
}

pub struct UserManager {
    path: PathBuf,
    current: usize,
    users: HashMap<usize, User>,
}

impl UserManager {
    pub fn create_at_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        fs::create_dir(&path)?;

        let current = 0;
        let user = User::create_under(&path)?;
        let mut users = HashMap::new();
        users.insert(current, user);

        Ok(Self {
            path: path.to_owned(),
            current,
            users,
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::PathNotFound(path.to_owned()));
        }

        todo!()
    }

    fn get_current(&self) -> &User {
        self.users
            .get(&self.current)
            .expect("The current user should always be present in the manager")
    }

    fn get_current_mut(&mut self) -> &mut User {
        self.users
            .get_mut(&self.current)
            .expect("The current user should always be present in the manager")
    }

    pub fn insert_package(&mut self, hash: &u64, store: &mut Store) -> Result<()> {
        self.get_current_mut()
            .generation_manager
            .insert_package(hash, store)
    }

    pub fn remove_generation(&mut self, id: usize) -> Result<()> {
        self.get_current_mut()
            .generation_manager
            .remove_generation(id)
    }

    pub fn link_global(&self, id: usize, global_paths: ComponentPaths) -> Result<()> {
        self.get_current()
            .generation_manager
            .link_global(id, global_paths)
    }

    pub fn link_current_global(&self, global_paths: ComponentPaths) -> Result<()> {
        self.get_current()
            .generation_manager
            .link_current_global(global_paths)
    }

    pub fn list_current_packages(&self) {
        self.get_current()
            .generation_manager
            .list_current_packages();
    }

    pub fn packages(&self) -> impl Iterator<Item = &u64> {
        self.users
            .iter()
            .flat_map(|(_id, user)| user.generation_manager.packages())
    }
}
