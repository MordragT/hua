use crate::extra::persist::Pot;
use crate::generation::GenerationManager;
use rustbreak::PathDatabase;
use snafu::ResultExt;
use std::fs;
use std::path::Path;

use super::*;

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

    pub fn generation_manager(&self) -> &GenerationManager {
        &self.generation_manager
    }

    pub fn generation_manager_mut(&mut self) -> &mut GenerationManager {
        &mut self.generation_manager
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn flush(self) -> UserResult<()> {
        self.database
            .put_data((self.name, self.generation_manager), true)
            .context(RustbreakSnafu {
                message: "could not save data".to_owned(),
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::User;
    use temp_dir::TempDir;

    #[test]
    fn user_create_under() {
        let temp_dir = TempDir::new().unwrap();

        let _user = User::init(temp_dir.path(), "example".to_owned()).unwrap();
        let name = users::get_current_username()
            .unwrap()
            .into_string()
            .unwrap();

        let user_dir = temp_dir.child(name);

        assert!(user_dir.exists());
        assert!(user_dir.is_dir());
    }
}
