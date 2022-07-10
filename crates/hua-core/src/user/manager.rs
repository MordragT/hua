use crate::{store::id::PackageId, user::User, GID, UID};
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

    /// Opens an old [UserManager] under the given [Path].
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

    /// Returns all packages respective [PackageId's](PackageId) inside all [users](User).
    pub fn packages(&self) -> impl Iterator<Item = &PackageId> {
        self.users.iter().map(|user| user.packages()).flatten()
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

    /// Borrow the current [User].
    pub fn current(&self) -> &User {
        &self.users[self.current]
    }

    /// Borrow the current [User] mutable.
    pub fn current_mut(&mut self) -> &mut User {
        &mut self.users[self.current]
    }
}

#[cfg(test)]
mod tests {
    use super::UserManager;
    use crate::{store::LocalStore, support::*};
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
        let _ = user_manager
            .current_mut()
            .insert_requirement(req, &store)
            .unwrap();

        let res = user_manager.contains(&id);

        assert!(res);
    }
}
