use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rustbreak::PathDatabase;
use serde::{Deserialize, Serialize};

use crate::generation::GenerationManager;
use crate::persist::Pot;
use crate::{error::*, ComponentPaths, Store};

pub const USERS_DB: &str = "users.db";

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Users {
    pub current: usize,
    pub list: HashMap<usize, User>,
}

impl Users {
    pub fn create_under<P: AsRef<Path>>(path: P) -> Result<Self> {
        let current = 0;
        let mut list = HashMap::new();
        let user = User::create_under(path.as_ref())?;
        list.insert(current, user);

        Ok(Self { current, list })
    }
}

pub struct UserManager {
    path: PathBuf,
    database: PathDatabase<Users, Pot>,
}

impl UserManager {
    pub fn create_at_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        fs::create_dir(&path)?;

        let database =
            PathDatabase::create_at_path(path.join(USERS_DB), Users::create_under(&path)?)?;

        Ok(Self {
            path: path.to_owned(),
            database,
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::PathNotFound(path.to_owned()));
        }

        let database = PathDatabase::load_from_path(path.join(USERS_DB))?;

        Ok(Self {
            path: path.to_owned(),
            database,
        })
    }

    fn read_current<R, T: FnOnce(&User) -> R>(&self, task: T) -> Result<R> {
        let res = self.database.read(|db| {
            let current = db
                .list
                .get(&db.current)
                .expect("The current user should always be present in the manager");
            task(current)
        })?;
        Ok(res)
    }

    fn write_current<R, T: FnOnce(&mut User) -> R>(&self, task: T) -> Result<R> {
        let res = self.database.write(|db| {
            let current = db
                .list
                .get_mut(&db.current)
                .expect("The current user should always be present in the manager");
            task(current)
        })?;
        Ok(res)
    }

    pub fn insert_package(&mut self, hash: &u64, store: &mut Store) -> Result<()> {
        self.write_current(|user| user.generation_manager.insert_package(hash, store))?
    }

    pub fn remove_generation(&mut self, id: usize) -> Result<()> {
        self.write_current(|user| user.generation_manager.remove_generation(id))?
    }

    pub fn link_global(&self, id: usize, global_paths: ComponentPaths) -> Result<()> {
        self.read_current(|user| user.generation_manager.link_global(id, global_paths))?
    }

    pub fn link_current_global(&self, global_paths: ComponentPaths) -> Result<()> {
        self.read_current(|user| user.generation_manager.link_current_global(global_paths))?
    }

    pub fn list_current_packages(&self) -> Result<()> {
        self.read_current(|user| user.generation_manager.list_current_packages())?;
        Ok(())
    }

    pub fn contains_package(&self, hash: &u64) -> Result<bool> {
        let res = self.database.read(|db| {
            db.list
                .iter()
                .flat_map(|(_id, user)| user.generation_manager.packages())
                .find(|key| *key == hash)
                .is_some()
        })?;
        Ok(res)
    }
}
