use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use rustbreak::PathDatabase;
use serde::{Deserialize, Serialize};

use crate::generation::GenerationManager;
use crate::persist::Pot;
use crate::{components::ComponentPaths, error::*, Store};

/// The filename of the users database
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

// TODO: use put_data and get_data and flush instead of db.read and db.write

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
    /// If the package was not present return true and create a new generation with the package.
    /// If it was present false is returned.
    pub fn insert_package(&mut self, index: usize, store: &mut Store) -> Result<bool> {
        self.write_current(|user| user.generation_manager.insert_package(index, store))?
    }

    /// Remove a package from the current user.
    /// If the package was present return true and create a new generation without the package.
    /// If it was not present false is returned.
    pub fn remove_package(&mut self, index: usize, store: &mut Store) -> Result<bool> {
        self.write_current(|user| user.generation_manager.remove_package(index, store))?
    }

    /// Removes the specified generation.
    /// Returns true if the generation was present and false if it wasnt.
    pub fn remove_generation(&mut self, id: usize) -> Result<bool> {
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
    pub fn contains_package(&self, index: usize) -> Result<bool> {
        // let res = self.database.read(|db| {
        //     db.list
        //         .iter()
        //         .flat_map(|(_id, user)| user.generation_manager.packages())
        //         .find(|key| *key == index)
        //         .is_some()
        // })?;
        // Ok(res)
        todo!()
    }

    /// Returns the path of the user manager
    pub fn path(&self) -> &Path {
        &self.path
    }

    // TODO maybe flush just after every io operation

    /// Flushes all data to the backend
    pub fn flush(&self) -> Result<()> {
        self.database.save()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};
    use std::fs::{self, File};

    use relative_path::RelativePathBuf;
    use semver::Version;
    use temp_dir::TempDir;

    use crate::components::ComponentPaths;
    use crate::{Binary, Component, Package, Store};

    use super::{User, UserManager};

    use super::USERS_DB;

    fn user_manager_insert_package(temp_dir: &TempDir) -> (UserManager, Store, usize) {
        let path = temp_dir.child("user");
        let store_path = temp_dir.child("store");
        let package_path = temp_dir.child("package");

        let package = {
            let package_bin_path = package_path.join("bin");
            fs::create_dir_all(&package_bin_path).unwrap();

            let shell_path = package_bin_path.join("package.sh");

            let _bin = File::create(&shell_path).unwrap();
            let mut provides = BTreeSet::new();

            provides.insert(Component::Binary(Binary::Shell(RelativePathBuf::from(
                "bin/package.sh",
            ))));

            Package::new("package", Version::new(1, 0, 0), provides, BTreeSet::new())
        };

        let mut store = Store::create_at_path(&store_path).unwrap();
        let index = store.insert(package, package_path).unwrap();

        let mut user_manager = UserManager::create_at_path(&path).unwrap();
        assert!(user_manager.insert_package(index, &mut store).unwrap());

        (user_manager, store, index)
    }

    #[test]
    fn user_create_under() {
        let temp_dir = TempDir::new().unwrap();

        let _user = User::create_under(temp_dir.path()).unwrap();
        let name = users::get_current_username()
            .unwrap()
            .into_string()
            .unwrap();

        let user_dir = temp_dir.child(name);

        assert!(user_dir.exists());
        assert!(user_dir.is_dir());
    }

    #[test]
    fn user_manager_create_at_path() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");

        let _user_manager = UserManager::create_at_path(&path).unwrap();

        let db = path.join(USERS_DB);

        assert!(db.exists());
        assert!(db.is_file());
    }

    #[test]
    fn user_manager_open_ok() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");

        {
            let _user_manager = UserManager::create_at_path(&path).unwrap();
        }

        let _user_manager = UserManager::open(&path).unwrap();
    }

    #[test]
    fn user_manager_open_err() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");

        let user_manager = UserManager::open(&path);
        assert!(user_manager.is_err());
    }

    #[test]
    fn user_manager_insert_package_true() {
        let temp_dir = TempDir::new().unwrap();
        let _ = user_manager_insert_package(&temp_dir);
    }

    #[test]
    fn user_manager_insert_package_false() {
        let temp_dir = TempDir::new().unwrap();
        let (mut user_manager, mut store, hash) = user_manager_insert_package(&temp_dir);
        assert!(!user_manager.insert_package(hash, &mut store).unwrap());
    }

    #[test]
    fn user_manager_remove_package_true() {
        let temp_dir = TempDir::new().unwrap();
        let (mut user_manager, mut store, hash) = user_manager_insert_package(&temp_dir);
        assert!(user_manager.remove_package(hash, &mut store).unwrap());
    }

    #[test]
    fn user_manager_remove_package_false() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");
        let store_path = temp_dir.child("store");

        let mut store = Store::create_at_path(&store_path).unwrap();
        let mut user_manager = UserManager::create_at_path(&path).unwrap();

        assert!(!user_manager.remove_package(129238423, &mut store).unwrap());
    }

    #[test]
    fn user_manager_remove_generation_true() {
        let temp_dir = TempDir::new().unwrap();
        let (mut user_manager, _, _) = user_manager_insert_package(&temp_dir);
        assert!(user_manager.remove_generation(0).unwrap());
    }

    #[test]
    fn user_manager_remove_generation_false() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(&path).unwrap();

        assert!(!user_manager.remove_generation(1).unwrap());
    }

    #[test]
    fn user_manager_remove_generation_err() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(&path).unwrap();

        assert!(user_manager.remove_generation(0).is_err());
    }

    #[test]
    fn user_manager_switch_global_links() {
        let temp_dir = TempDir::new().unwrap();
        let (mut user_manager, _, _) = user_manager_insert_package(&temp_dir);

        let global_temp_path = temp_dir.child("global");
        fs::create_dir(&global_temp_path).unwrap();

        let global_paths = ComponentPaths::from_path(global_temp_path);
        global_paths.create_dirs().unwrap();

        user_manager.switch_global_links(&global_paths).unwrap();

        let package_bin_link = global_paths.binary.join("package.sh");
        assert!(package_bin_link.exists());
        assert!(package_bin_link.is_symlink());
    }

    #[test]
    fn user_manager_contains_package_false() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");

        let user_manager = UserManager::create_at_path(&path).unwrap();
        let res = user_manager.contains_package(123804230842).unwrap();

        assert!(!res);
    }

    #[test]
    fn user_manager_contains_package_true() {
        let temp_dir = TempDir::new().unwrap();
        let (user_manager, _, hash) = user_manager_insert_package(&temp_dir);

        let res = user_manager.contains_package(hash).unwrap();

        assert!(res);
    }
}
