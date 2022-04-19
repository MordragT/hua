use crate::{
    dependency::Requirement,
    extra::{path::ComponentPathBuf, persist::Pot},
    generation::GenerationManager,
    store::{backend::ReadBackend, id::PackageId, Store},
    user::User,
};
use rustbreak::{Database, PathDatabase};
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{
    fs,
    os::unix::prelude::PermissionsExt,
    path::{Path, PathBuf},
};

use super::*;

/// The filename of the users database
pub const USERS_DB: &str = "users.db";

// TODO: instead of allowing the db to be written by everybody
// ditch the db all together and implement a file based solution

// TODO change Users to simple Vec as the current user
// must not be saved inside the database (it can change all the time anyway)

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Users {
    pub current: usize,
    pub list: Vec<User>,
}

impl Users {
    pub fn init<P: AsRef<Path>>(path: P) -> UserResult<Self> {
        let current = 0;
        let mut list = Vec::new();

        let name = users::get_current_username()
            .ok_or(UserError::UsernameNotFound)?
            .into_string()
            .map_err(|_| UserError::Whatever {
                message: "OsString conversion error".to_owned(),
            })?;

        let path = path.as_ref().join(&name);
        fs::create_dir(&path).context(IoSnafu)?;

        let user = User::init(path, name)?;
        list.push(user);

        Ok(Self { current, list })
    }

    pub fn insert<P: AsRef<Path>>(&mut self, name: String, path: P) -> UserResult<()> {
        let path = path.as_ref().join(&name);
        fs::create_dir(&path).context(IoSnafu)?;

        let user = User::init(path, name)?;

        let current = self.list.len();
        self.list.push(user);
        self.current = current;

        Ok(())
    }

    pub fn find(&self, name: &String) -> Option<usize> {
        self.list
            .iter()
            .enumerate()
            .find_map(|(n, user)| if user.name() == name { Some(n) } else { None })
    }

    pub fn change_current_if_found(&mut self, name: &String) -> bool {
        if let Some(index) = self.find(name) {
            self.current = index;
            true
        } else {
            false
        }
    }

    pub fn current_user(&self) -> &User {
        &self.list[self.current]
    }

    pub fn current_user_mut(&mut self) -> &mut User {
        &mut self.list[self.current]
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
    pub fn init<P: AsRef<Path>>(path: P) -> UserResult<Self> {
        let path = path.as_ref();
        fs::create_dir(&path).context(IoSnafu)?;
        let mut perm = fs::metadata(path).context(IoSnafu)?.permissions();
        perm.set_mode(0o777);
        fs::set_permissions(path, perm).context(IoSnafu)?;

        let db_path = path.join(USERS_DB);
        let database = PathDatabase::create_at_path(db_path.clone(), Users::init(&path)?).context(
            RustbreakSnafu {
                message: format!("at {USERS_DB}"),
            },
        )?;

        let mut perm = fs::metadata(&db_path).context(IoSnafu)?.permissions();
        perm.set_mode(0o666);
        fs::set_permissions(db_path, perm).context(IoSnafu)?;

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

        let database: Database<Users, _, _> = PathDatabase::load_from_path(path.join(USERS_DB))
            .context(RustbreakSnafu {
                message: format!("at {USERS_DB}"),
            })?;

        let users = {
            let mut users = database.get_data(false).context(RustbreakSnafu {
                message: "could not load user data".to_owned(),
            })?;
            let current_name = users::get_current_username()
                .ok_or(UserError::UsernameNotFound)?
                .into_string()
                .map_err(|_| UserError::Whatever {
                    message: "OsString conversion error".to_owned(),
                })?;
            if users.current_user().name() != &current_name
                && !users.change_current_if_found(&current_name)
            {
                users.insert(current_name, path)?;
                database.put_data(users, true).context(RustbreakSnafu {
                    message: "Put user data error".to_owned(),
                })?;
                database.get_data(true).context(RustbreakSnafu {
                    message: "Get user data error".to_owned(),
                })?
            } else {
                users
            }
        };

        Ok(Self {
            path: path.to_owned(),
            database,
            users,
        })
    }

    fn current_generation_manager(&self) -> &GenerationManager {
        self.users.current_user().generation_manager()
    }

    fn current_generation_manager_mut(&mut self) -> &mut GenerationManager {
        self.users.current_user_mut().generation_manager_mut()
    }

    /// Inserts a requiremnt into the current user.
    /// If the requirement was not fullfilled, a try to get a matching package from the store is started.
    /// If the package could be retrieved a new generation is created and true is returned.
    /// If it fullfilled false is returned
    pub fn insert_requirement<B: ReadBackend<Source = PathBuf>>(
        &mut self,
        requirement: Requirement,
        store: &Store<PathBuf, B>,
    ) -> UserResult<bool> {
        Ok(self
            .current_generation_manager_mut()
            .insert_requirement(requirement, store)
            .context(GenerationSnafu)?)
    }

    /// Remove a package from the current user.
    /// If the package was present return true and create a new generation without the package.
    /// If it was not present false is returned.
    pub fn remove_requirement<B: ReadBackend<Source = PathBuf>>(
        &mut self,
        requirement: &Requirement,
        store: &Store<PathBuf, B>,
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
            .map(|user| user.generation_manager().packages())
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
    use crate::user::UserError;
    use crate::{store::LocalStore, support::*};
    use std::assert_matches::assert_matches;
    use std::fs;
    use temp_dir::TempDir;

    #[test]
    fn user_manager_create_at_path() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");

        let _user_manager = UserManager::init(&path).unwrap();

        let db = path.join(USERS_DB);

        assert!(db.exists());
        assert!(db.is_file());
    }

    #[test]
    fn user_manager_open_ok() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");

        {
            let _user_manager = UserManager::init(&path).unwrap();
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
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

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
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

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
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let store = LocalStore::init(&store_path).unwrap();

        let req = req("one", ">0.0.1");
        let res = user_manager.insert_requirement(req, &store).unwrap_err();

        assert_matches!(res, UserError::GenerationError { source: _ });
    }

    #[test]
    fn user_manager_remove_requirement_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

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
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let store = LocalStore::init(&store_path).unwrap();

        let req = req("one", ">0.0.1");
        let res = user_manager.remove_requirement(&req, &store).unwrap();

        assert!(!res);
    }

    #[test]
    fn user_manager_remove_generation_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

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
        let mut user_manager = UserManager::init(path).unwrap();
        let res = user_manager.remove_generation(2).unwrap();

        assert!(!res);
    }

    #[test]
    fn user_manager_remove_generation_err() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");
        let mut user_manager = UserManager::init(&path).unwrap();

        assert!(user_manager.remove_generation(0).is_err());
    }

    #[test]
    fn user_manager_switch_global_links() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

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
        let user_manager = UserManager::init(path).unwrap();

        let res = user_manager.contains(&[1_u8; 32].into());

        assert!(!res);
    }

    #[test]
    fn user_manager_contains_package_index_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

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
