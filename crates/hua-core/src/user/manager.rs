use crate::{
    dependency::Requirement,
    extra::path::ComponentPathBuf,
    generation::GenerationManager,
    store::{backend::ReadBackend, id::PackageId, Store},
    user::User,
    GID, UID,
};
use log::debug;
use snafu::ResultExt;
use std::{
    fs,
    os::unix::{self, prelude::PermissionsExt},
    path::{Path, PathBuf},
};

use super::*;

/// Manages all users.
pub struct UserManager {
    path: PathBuf,
    current: usize,
    users: Vec<User>,
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
        unix::fs::chown(&path, UID, GID).context(IoSnafu)?;

        debug!("User manager path created at {path:?}");

        let current = 0;
        let mut list = Vec::new();

        let name = users::get_current_username()
            .ok_or(UserError::UsernameNotFound)?
            .into_string()
            .map_err(|_| UserError::Whatever {
                message: "OsString conversion error".to_owned(),
            })?;

        let user = User::init(path.join(&name), name)?;
        list.push(user);

        debug!("Current user added to user manager");

        Ok(Self {
            path: path.to_owned(),
            users: list,
            current,
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

        let mut users = Vec::new();

        for entry in path.read_dir().context(IoSnafu)? {
            let entry = entry.context(IoSnafu)?;
            let user = User::open(entry.path())?;
            users.push(user);
        }

        let current = {
            let current_name = users::get_current_username()
                .ok_or(UserError::UsernameNotFound)?
                .into_string()
                .map_err(|_| UserError::Whatever {
                    message: "OsString conversion error".to_owned(),
                })?;

            if let Some((n, _user)) = users
                .iter()
                .enumerate()
                .find(|(_n, user)| user.name() == &current_name)
            {
                n
            } else {
                let path = path.join(&current_name);
                let user = User::init(path, current_name)?;
                let current = users.len();
                users.push(user);
                current
            }
        };

        Ok(Self {
            path: path.to_owned(),
            current,
            users,
        })
    }

    fn current_user(&self) -> &User {
        &self.users[self.current]
    }

    fn current_user_mut(&mut self) -> &mut User {
        &mut self.users[self.current]
    }

    fn current_generation_manager(&self) -> &GenerationManager {
        self.current_user().generation_manager()
    }

    fn current_generation_manager_mut(&mut self) -> &mut GenerationManager {
        self.current_user_mut().generation_manager_mut()
    }

    pub fn current_generation_index(&self) -> usize {
        self.current_user().generation_manager().current_index()
    }

    /// Inserts a requiremnt into the current user.
    /// If the requirement was not fullfilled, a try to get a matching package from the store is started.
    /// If the package could be retrieved a new generation is created and true is returned.
    /// If it fullfilled false is returned
    pub fn insert_requirement<B: ReadBackend<Source = PathBuf>>(
        &mut self,
        requirement: Requirement,
        store: &Store<PathBuf, B>,
        global_paths: &ComponentPathBuf,
    ) -> UserResult<bool> {
        Ok(self
            .current_generation_manager_mut()
            .insert_requirement(requirement, store, global_paths)
            .context(GenerationSnafu)?)
    }

    /// Remove a package from the current user.
    /// If the package was present return true and create a new generation without the package.
    /// If it was not present false is returned.
    pub fn remove_requirement<B: ReadBackend<Source = PathBuf>>(
        &mut self,
        requirement: &Requirement,
        store: &Store<PathBuf, B>,
        global_paths: &ComponentPathBuf,
    ) -> UserResult<bool> {
        Ok(self
            .current_generation_manager_mut()
            .remove_requirement(requirement, store, global_paths)
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

    pub fn filter_requirements_by_name_containing<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a Requirement> {
        self.filter_requirements(move |req| req.name().contains(name))
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
    fn switch_global_links(&mut self, global_paths: &ComponentPathBuf) -> UserResult<()> {
        Ok(self
            .current_generation_manager_mut()
            .switch_global_links(global_paths)
            .context(GenerationSnafu)?)
    }

    pub fn switch_generation(
        &mut self,
        id: usize,
        global_paths: &ComponentPathBuf,
    ) -> UserResult<()> {
        self.current_generation_manager_mut()
            .switch_to(id, global_paths)
            .context(GenerationSnafu)?;
        Ok(())
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

    /// Flushes all data to the backend
    pub fn flush(self) -> UserResult<()> {
        for user in self.users {
            user.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::UserManager;
    use crate::extra::hash;
    use crate::extra::path::ComponentPathBuf;
    use crate::user::UserError;
    use crate::{store::LocalStore, support::*};
    use std::assert_matches::assert_matches;
    use std::fs;
    use temp_dir::TempDir;

    // #[test]
    // fn user_manager_create_at_path() {
    //     let temp_dir = TempDir::new().unwrap();
    //     let path = temp_dir.child("user");

    //     let _user_manager = UserManager::init(&path).unwrap();

    //     let db = path.join(USERS_DB);

    //     assert!(db.exists());
    //     assert!(db.is_file());
    // }

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
        store.insert(one).unwrap();

        let global_temp = temp_dir.child("global");
        fs::create_dir(&global_temp).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp);
        global_paths.create_dirs(false).unwrap();

        let req = req("one", ">0.0.1");
        let res = user_manager
            .insert_requirement(req, &store, &global_paths)
            .unwrap();

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
        store.insert(one).unwrap();

        let global_temp = temp_dir.child("global");
        fs::create_dir(&global_temp).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp);
        global_paths.create_dirs(false).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager
            .insert_requirement(req.clone(), &store, &global_paths)
            .unwrap();
        let res = user_manager
            .insert_requirement(req, &store, &global_paths)
            .unwrap();

        assert!(!res);
    }

    #[test]
    fn user_manager_insert_requirement_err() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let store = LocalStore::init(&store_path).unwrap();

        let global_temp = temp_dir.child("global");
        fs::create_dir(&global_temp).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp);
        global_paths.create_dirs(false).unwrap();

        let req = req("one", ">0.0.1");
        let res = user_manager
            .insert_requirement(req, &store, &global_paths)
            .unwrap_err();

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
        store.insert(one).unwrap();

        let global_temp = temp_dir.child("global");
        fs::create_dir(&global_temp).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp);
        global_paths.create_dirs(false).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager
            .insert_requirement(req.clone(), &store, &global_paths)
            .unwrap();
        let res = user_manager
            .remove_requirement(&req, &store, &global_paths)
            .unwrap();

        assert!(res);
    }

    #[test]
    fn user_manager_remove_requirement_false() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user_manager = UserManager::init(path).unwrap();

        let store_path = temp_dir.child("store");
        let store = LocalStore::init(&store_path).unwrap();

        let global_temp = temp_dir.child("global");
        fs::create_dir(&global_temp).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp);
        global_paths.create_dirs(false).unwrap();

        let req = req("one", ">0.0.1");
        let res = user_manager
            .remove_requirement(&req, &store, &global_paths)
            .unwrap();

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
        store.insert(one).unwrap();

        let global_temp = temp_dir.child("global");
        fs::create_dir(&global_temp).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp);
        global_paths.create_dirs(false).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager
            .insert_requirement(req, &store, &global_paths)
            .unwrap();

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
        store.insert(one).unwrap();

        let global_temp = temp_dir.child("global");
        fs::create_dir(&global_temp).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp);
        global_paths.create_dirs(false).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager
            .insert_requirement(req, &store, &global_paths)
            .unwrap();

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
        let one_id = hash::root_hash(&one.path, &one.name()).unwrap();
        let _ = store.insert(one).unwrap();

        let global_temp = temp_dir.child("global");
        fs::create_dir(&global_temp).unwrap();
        let global_paths = ComponentPathBuf::from_path(&global_temp);
        global_paths.create_dirs(false).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user_manager
            .insert_requirement(req, &store, &global_paths)
            .unwrap();

        let res = user_manager.contains(&one_id);

        assert!(res);
    }
}
