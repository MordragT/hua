use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rustbreak::PathDatabase;
use serde::{Deserialize, Serialize};

use crate::generation::GenerationManager;
use crate::persist::Pot;
use crate::{error::*, extra::ComponentPaths, Store};

pub const USERS_DB: &str = "users.db";

/// A single User who owns several generations.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    name: String,
    path: PathBuf,
    generation_manager: GenerationManager,
}

impl User {
    /// Create a new user directory under the given path.
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

/// Manages all users.
pub struct UserManager {
    path: PathBuf,
    database: PathDatabase<Users, Pot>,
}

impl UserManager {
    /// Create a new user manager under the given path.
    /// Returns an error if the path is already present.
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

    /// Opens an old user manager under the given path.
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

    /// Inserts a package into the current user.
    /// By doing that a new generation is created.
    pub fn insert_package(&mut self, hash: &u64, store: &mut Store) -> Result<()> {
        self.write_current(|user| user.generation_manager.insert_package(hash, store))?
    }

    /// Removes a package.
    /// By doing so, a new generation without the package is created.
    pub fn remove_package(&mut self, hash: &u64, store: &mut Store) -> Result<()> {
        self.write_current(|user| user.generation_manager.remove_package(hash, store))?
    }

    /// Removes the specified generation.
    pub fn remove_generation(&mut self, id: usize) -> Result<()> {
        self.write_current(|user| user.generation_manager.remove_generation(id))?
    }

    /// Unlinks the old generation and links the new one globally
    pub fn switch_global_links(&mut self, global_paths: &ComponentPaths) -> Result<()> {
        self.write_current(|user| user.generation_manager.switch_global_links(global_paths))?
    }

    /// Lists all packages in the current generation.
    pub fn list_current_packages(&self) -> Result<()> {
        self.read_current(|user| user.generation_manager.list_current_packages())?;
        Ok(())
    }

    /// Lists all generations
    pub fn list_current_generations(&self) -> Result<()> {
        self.read_current(|user| user.generation_manager.list_generations())?;
        Ok(())
    }

    /// Checks wether the package is stored inside any generation of all users.
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

    // TODO maybe flush just after every io operation

    /// Flushes all data to the backend
    pub fn flush(&self) -> Result<()> {
        self.database.save()?;
        Ok(())
    }
}
