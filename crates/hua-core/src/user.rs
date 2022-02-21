use std::fs;
use std::path::{Path, PathBuf};

use crate::error::*;
use crate::generation::GenerationManager;

#[derive(Debug)]
pub struct User {
    pub name: String,
    path: PathBuf,
    pub generation_manager: GenerationManager,
}

impl User {
    pub fn create_under<P: AsRef<Path>>(path: P) -> Result<Self> {
        let name = users::get_current_username()
            .ok_or(Error::UsernameNotFound)?
            .into_string()?;

        let path = path.as_ref().join(&name);
        fs::create_dir(&path)?;

        let generation_manager = GenerationManager::create_under(&path)?;

        Ok(Self {
            name,
            path,
            generation_manager,
        })
    }
}
