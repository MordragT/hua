use crate::{
    dependency::Requirement,
    extra::{path::ComponentPathBuf, persist::Pot},
    generation::GenerationManager,
    store::{Backend, PackageId, Store},
    user::User,
};
use rustbreak::PathDatabase;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use super::*;

/// The filename of the users database
pub const USERS_DB: &str = "users.db";

// TODO: use put_data and get_data and flush instead of db.read and db.write

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Users {
    pub current: usize,
    pub list: HashMap<usize, User>,
}

impl Users {
    pub fn create_under<P: AsRef<Path>>(path: P) -> UserResult<Self> {
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
    users: Users,
}

impl UserManager {
    /// Create a new user manager under the given path.
    /// Returns an error if the path is already present.
    pub fn create_at_path<P: AsRef<Path>>(path: P) -> UserResult<Self> {
        let path = path.as_ref();
        fs::create_dir(&path).context(IoSnafu)?;

        let database =
            PathDatabase::create_at_path(path.join(USERS_DB), Users::create_under(&path)?)
                .context(RustbreakSnafu {
                    message: format!("at {USERS_DB}"),
                })?;

        let users = database.get_data(false).context(RustbreakSnafu {
            message: "could not load user data".to_owned(),
        })?;

        Ok(Self {
            path: path.to_owned(),
            database,
            users,
        })
    }

    /// Opens an old user manager under the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> UserResult<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(UserError::PathNotExisting {
                path: path.to_owned(),
            });
        }

        let database =
            PathDatabase::load_from_path(path.join(USERS_DB)).context(RustbreakSnafu {
                message: format!("at {USERS_DB}"),
            })?;

        let users = database.get_data(false).context(RustbreakSnafu {
            message: "could not load user data".to_owned(),
        })?;

        Ok(Self {
            path: path.to_owned(),
            database,
            users,
        })
    }

    fn current_generation_manager(&self) -> &GenerationManager {
        unsafe { &self.users.list.get(&self.users.current).unwrap_unchecked() }.generation_manager()
    }

    fn current_generation_manager_mut(&mut self) -> &mut GenerationManager {
        unsafe {
            self.users
                .list
                .get_mut(&self.users.current)
                .unwrap_unchecked()
        }
        .generation_manager_mut()
    }

    /// Inserts a requiremnt into the current user.
    /// If the requirement was not fullfilled, a try to get a matching package from the store is started.
    /// If the package could be retrieved a new generation is created and true is returned.
    /// If it fullfilled false is returned
    pub fn insert_requirement<B: Backend>(
        &mut self,
        requirement: Requirement,
        store: &Store<B>,
    ) -> UserResult<bool> {
        Ok(self
            .current_generation_manager_mut()
            .insert_requirement(requirement, store)
            .context(GenerationSnafu)?)
    }

    /// Remove a package from the current user.
    /// If the package was present return true and create a new generation without the package.
    /// If it was not present false is returned.
    pub fn remove_requirement<B: Backend>(
        &mut self,
        requirement: &Requirement,
        store: &Store<B>,
    ) -> UserResult<bool> {
        Ok(self
            .current_generation_manager_mut()
            .remove_requirement(requirement, store)
            .context(GenerationSnafu)?)
    }

    pub fn filter_requirements<'a, P>(
        &'a self,
        predicate: P,
    ) -> impl Iterator<Item = &'a Requirement>
    where
        P: Fn(&Requirement) -> bool + 'a,
    {
        self.current_generation_manager()
            .current_requirements()
            .iter()
            .filter(move |req| predicate(req))
    }

    pub fn filter_requirements_by_name_starting_with<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a Requirement> {
        self.filter_requirements(move |req| req.name().starts_with(name))
    }

    /// Removes the specified generation.
    /// Returns true if the generation was present and false if it wasnt.
    pub fn remove_generation(&mut self, id: usize) -> UserResult<bool> {
        Ok(self
            .current_generation_manager_mut()
            .remove_generation(id)
            .context(GenerationSnafu)?)
    }

    /// Unlinks the old generation and links the new one globally
    pub fn switch_global_links(&mut self, global_paths: &ComponentPathBuf) -> UserResult<()> {
        Ok(self
            .current_generation_manager_mut()
            .switch_global_links(global_paths)
            .context(GenerationSnafu)?)
    }

    /// Lists all packages in the current generation.
    pub fn list_current_packages(&self) {
        self.current_generation_manager().list_current_packages();
    }

    /// Lists all generations
    pub fn list_current_generations(&self) {
        self.current_generation_manager().list_generations();
    }

    pub fn packages(&self) -> impl Iterator<Item = &PackageId> {
        self.users
            .list
            .iter()
            .map(|(_, user)| user.generation_manager().packages())
            .flatten()
    }

    /// Checks wether the package is stored inside any generation of all users.
    pub fn contains(&self, index: &PackageId) -> bool {
        self.packages().find(|idx| *idx == index).is_some()
    }

    /// Returns the path of the user manager
    pub fn path(&self) -> &Path {
        &self.path
    }

    // TODO maybe flush just after every io operation

    /// Flushes all data to the backend
    pub fn flush(self) -> UserResult<()> {
        self.database
            .put_data(self.users, true)
            .context(RustbreakSnafu {
                message: "could not flush user data".to_owned(),
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::UserManager;
    use super::USERS_DB;
    use crate::extra::path::ComponentPathBuf;
    use crate::store::LocalBackend;
    use crate::user::UserError;
    use crate::{store::Store, support::*};
    use std::assert_matches::assert_matches;
    use std::fs;
    use temp_dir::TempDir;

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
    fn user_manager_insert_requirement_ok_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = Store::<LocalBackend>::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let res = user_manager.insert_requirement(req, &store).unwrap();

        assert!(res);
    }

    #[test]
    fn user_manager_insert_requirement_ok_false() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = Store::<LocalBackend>::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager
            .insert_requirement(req.clone(), &store)
            .unwrap();
        let res = user_manager.insert_requirement(req, &store).unwrap();

        assert!(!res);
    }

    #[test]
    fn user_manager_insert_requirement_err() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(path).unwrap();

        let store_path = temp_dir.child("store");
        let store = Store::<LocalBackend>::init(&store_path).unwrap();

        let req = req("one", ">0.0.1");
        let res = user_manager.insert_requirement(req, &store).unwrap_err();

        assert_matches!(res, UserError::GenerationError { source: _ });
    }

    #[test]
    fn user_manager_remove_requirement_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = Store::<LocalBackend>::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager
            .insert_requirement(req.clone(), &store)
            .unwrap();
        let res = user_manager.remove_requirement(&req, &store).unwrap();

        assert!(res);
    }

    #[test]
    fn user_manager_remove_requirement_false() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(path).unwrap();

        let store_path = temp_dir.child("store");
        let store = Store::<LocalBackend>::init(&store_path).unwrap();

        let req = req("one", ">0.0.1");
        let res = user_manager.remove_requirement(&req, &store).unwrap();

        assert!(!res);
    }

    #[test]
    fn user_manager_remove_generation_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = Store::<LocalBackend>::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager.insert_requirement(req, &store).unwrap();

        let res = user_manager.remove_generation(0).unwrap();

        assert!(res);
    }

    #[test]
    fn user_manager_remove_generation_false() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(path).unwrap();
        let res = user_manager.remove_generation(2).unwrap();

        assert!(!res);
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

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = Store::<LocalBackend>::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager.insert_requirement(req, &store).unwrap();

        let global_path = temp_dir.child("global");
        fs::create_dir(&global_path).unwrap();

        let global_paths = ComponentPathBuf::from_path(global_path);
        global_paths.create_dirs().unwrap();

        user_manager.switch_global_links(&global_paths).unwrap();

        let pkg_file = global_paths.library.join("one.so");
        assert!(pkg_file.exists());
        assert!(pkg_file.is_symlink());
    }

    #[test]
    fn user_manager_contains_package_index_false() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let user_manager = UserManager::create_at_path(path).unwrap();

        let res = user_manager.contains(&[1_u8; 32].into());

        assert!(!res);
    }

    #[test]
    fn user_manager_contains_package_index_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::create_at_path(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = Store::<LocalBackend>::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        let id = one.id;
        let _ = store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager.insert_requirement(req, &store).unwrap();

        let res = user_manager.contains(&id);

        assert!(res);
    }
}
