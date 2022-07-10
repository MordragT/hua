use super::*;
use crate::{
    dependency::Requirement,
    extra::{path::ComponentPathBuf, persist::Pot},
    generation::GenerationManager,
    store::{backend::ReadBackend, id::PackageId, Store},
};
use console::style;
use rustbreak::PathDatabase;
use snafu::ResultExt;
use std::{fmt, fs, path::Path};

const USER_DB: &str = "user.db";

// TODO change rustbreak error

/// A single User who owns several generations.
#[derive(Debug)]
pub struct User {
    name: String,
    generation_manager: GenerationManager,
    database: PathDatabase<(String, GenerationManager), Pot>,
}

impl User {
    /// Create a new user directory under the given path.
    pub fn init<P: AsRef<Path>>(path: P, name: String) -> UserResult<Self> {
        let path = path.as_ref();
        fs::create_dir(&path).context(IoSnafu)?;

        let generation_manager =
            GenerationManager::init(path.join("generations")).context(GenerationSnafu)?;
        let database = PathDatabase::create_at_path(path.join(USER_DB), (name, generation_manager))
            .context(RustbreakSnafu {
                message: "Could not create user database".to_owned(),
            })?;

        let (name, generation_manager) = database.get_data(false).context(RustbreakSnafu {
            message: "could not laod user database".to_owned(),
        })?;

        Ok(Self {
            name,
            generation_manager,
            database,
        })
    }

    /// Opens an existing [User] under the given [Path].
    pub fn open<P: AsRef<Path>>(path: P) -> UserResult<Self> {
        let path = path.as_ref();

        let database =
            PathDatabase::load_from_path(path.join(USER_DB)).context(RustbreakSnafu {
                message: "Could not load user database".to_owned(),
            })?;

        let (name, generation_manager) = database.get_data(false).context(RustbreakSnafu {
            message: "Could not get data from user database",
        })?;

        Ok(Self {
            name,
            generation_manager,
            database,
        })
    }

    #[deprecated]
    /// Returns a reference to the user's [GenerationManager].
    pub fn generation_manager(&self) -> &GenerationManager {
        &self.generation_manager
    }

    #[deprecated]
    /// Returns a mutable reference to the user's [GenerationManager].
    pub fn generation_manager_mut(&mut self) -> &mut GenerationManager {
        &mut self.generation_manager
    }

    /// Inserts a requiremnt into the user.
    /// If the requirement was not fullfilled, a try to get a matching package from the store is started.
    /// If the package could be retrieved a new generation is created and true is returned.
    /// If it fullfilled false is returned
    pub fn insert_requirement<B: ReadBackend<Source = PathBuf>>(
        &mut self,
        requirement: Requirement,
        store: &Store<PathBuf, B>,
    ) -> UserResult<bool> {
        Ok(self
            .generation_manager
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
            .generation_manager
            .remove_requirement(requirement, store)
            .context(GenerationSnafu)?)
    }

    /// Filters all [requirements](Requirement) by the specified `predicate`.
    pub fn filter_requirements<'a, P>(
        &'a self,
        predicate: P,
    ) -> impl Iterator<Item = &'a Requirement>
    where
        P: Fn(&Requirement) -> bool + 'a,
    {
        self.generation_manager
            .current()
            .requirements()
            .iter()
            .filter(move |req| predicate(req))
    }

    /// Filters all [requirements](Requirement) by the specified `name`.
    pub fn filter_requirements_by_name_starting_with<'a>(
        &'a self,
        name: &'a str,
    ) -> impl Iterator<Item = &'a Requirement> {
        self.filter_requirements(move |req| req.name().starts_with(name))
    }

    /// Filters all [requirements](Requirement) by the specified `name`.
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
            .generation_manager
            .remove_generation(id)
            .context(GenerationSnafu)?)
    }

    /// Unlinks the old generation and links the new one globally
    pub fn switch_global_links(&mut self, global_paths: &ComponentPathBuf) -> UserResult<()> {
        Ok(self
            .generation_manager
            .switch_global_links(global_paths)
            .context(GenerationSnafu)?)
    }

    /// Switch to another Generation specified by the `id`
    pub fn switch_generation(
        &mut self,
        id: usize,
        global_paths: &ComponentPathBuf,
    ) -> UserResult<()> {
        self.generation_manager
            .switch_to(id, global_paths)
            .context(GenerationSnafu)?;
        Ok(())
    }

    /// Returns an [Iterator] of every [PackageId] respective [Package](crate::store::Package) inside any
    /// [Generation](crate::generation::Generation) of the user.
    pub fn packages(&self) -> impl Iterator<Item = &PackageId> {
        self.generation_manager.packages()
    }

    /// Returns the name of the [User].
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Flushes the in-memory changes to the backend.
    pub fn flush(self) -> UserResult<()> {
        self.database
            .put_data((self.name, self.generation_manager), true)
            .context(RustbreakSnafu {
                message: "could not save data".to_owned(),
            })?;
        Ok(())
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}:\n{}",
            style("User").green(),
            style(&self.name).blue(),
            &self.generation_manager
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_matches::assert_matches, fs};

    use crate::{
        extra::path::ComponentPathBuf,
        store::LocalStore,
        support::{pkg, req},
        user::UserError,
    };

    use super::{User, USER_DB};
    use temp_dir::TempDir;

    #[test]
    fn user_init() {
        let temp_dir = TempDir::new().unwrap();
        let user_dir = temp_dir.child("example");

        let _user = User::init(&user_dir, "example".to_owned()).unwrap();

        let user_db = user_dir.join(USER_DB);
        let generations_dir = user_dir.join("generations");

        assert!(user_dir.exists());
        assert!(user_dir.is_dir());

        assert!(user_db.exists());
        assert!(user_db.is_file());

        assert!(generations_dir.exists());
        assert!(generations_dir.is_dir());
    }

    #[test]
    fn user_insert_requirement_ok_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user = User::init(path, "user".to_owned()).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let res = user.insert_requirement(req, &store).unwrap();

        assert!(res);
    }

    #[test]
    fn user_insert_requirement_ok_false() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user = User::init(path, "user".to_owned()).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user.insert_requirement(req.clone(), &store).unwrap();
        let res = user.insert_requirement(req, &store).unwrap();

        assert!(!res);
    }

    #[test]
    fn user_insert_requirement_err() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user = User::init(path, "user".to_owned()).unwrap();

        let store_path = temp_dir.child("store");
        let store = LocalStore::init(&store_path).unwrap();

        let req = req("one", ">0.0.1");
        let res = user.insert_requirement(req, &store).unwrap_err();

        assert_matches!(res, UserError::GenerationError { source: _ });
    }

    #[test]
    fn user_remove_requirement_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user = User::init(path, "user".to_owned()).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user.insert_requirement(req.clone(), &store).unwrap();
        let res = user.remove_requirement(&req, &store).unwrap();

        assert!(res);
    }

    #[test]
    fn user_remove_requirement_false() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user = User::init(path, "user".to_owned()).unwrap();

        let store_path = temp_dir.child("store");
        let store = LocalStore::init(&store_path).unwrap();

        let req = req("one", ">0.0.1");
        let res = user.remove_requirement(&req, &store).unwrap();

        assert!(!res);
    }

    #[test]
    fn user_remove_generation_true() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user = User::init(path, "user".to_owned()).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user.insert_requirement(req, &store).unwrap();

        let res = user.remove_generation(0).unwrap();

        assert!(res);
    }

    #[test]
    fn user_remove_generation_false() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user = User::init(path, "user".to_owned()).unwrap();
        let res = user.remove_generation(2).unwrap();

        assert!(!res);
    }

    #[test]
    fn user_remove_generation_err() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("user");
        let mut user = User::init(&path, "user".to_owned()).unwrap();

        assert!(user.remove_generation(0).is_err());
    }

    #[test]
    fn user_switch_global_links() {
        let temp_dir = TempDir::new().unwrap();

        let path = temp_dir.child("user");
        let mut user = User::init(path, "user".to_owned()).unwrap();

        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(&store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        store.insert(one, one_path).unwrap();

        let req = req("one", ">0.0.1");
        let _ = user.insert_requirement(req, &store).unwrap();

        let global_path = temp_dir.child("global");
        fs::create_dir(&global_path).unwrap();

        let global_paths = ComponentPathBuf::from_path(global_path);
        global_paths.create_dirs(true).unwrap();

        user.switch_global_links(&global_paths).unwrap();

        let pkg_file = global_paths.library.join("one.so");
        assert!(pkg_file.exists());
        assert!(pkg_file.is_symlink());
    }
}
