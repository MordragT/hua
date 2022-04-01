use crate::generation::GenerationManager;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

/// A single User who owns several generations.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    name: String,
    path: PathBuf,
    generation_manager: GenerationManager,
}

impl User {
    /// Create a new user directory under the given path.
    pub fn create_under<P: AsRef<Path>>(path: P) -> UserResult<Self> {
        let name = users::get_current_username()
            .ok_or(UserError::UsernameNotFound)?
            .into_string()
            .map_err(|_| UserError::Whatever {
                message: "OsString conversion error".to_owned(),
            })?;

        let path = path.as_ref().join(&name);
        if !path.exists() {
            fs::create_dir(&path).context(IoSnafu)?;
        }

        let generation_manager = GenerationManager::create_under(&path).context(GenerationSnafu)?;

        Ok(Self {
            name,
            path,
            generation_manager,
        })
    }

    pub fn generation_manager(&self) -> &GenerationManager {
        &self.generation_manager
    }

    pub fn generation_manager_mut(&mut self) -> &mut GenerationManager {
        &mut self.generation_manager
    }
}

#[cfg(test)]
mod tests {
    use super::User;
    use temp_dir::TempDir;

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
}