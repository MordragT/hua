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
    pub fn init<P: AsRef<Path>>(path: P, name: String) -> UserResult<Self> {
        let path = path.as_ref().join("generations");
        fs::create_dir(&path).context(IoSnafu)?;

        let generation_manager = GenerationManager::init(&path).context(GenerationSnafu)?;

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

    pub fn name(&self) -> &String {
        &self.name
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
