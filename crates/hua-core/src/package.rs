use crate::{error::*, OptionalComponentPaths};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

impl Package {
    pub fn new<P: AsRef<Path>>(name: &str, version: &str, path: P) -> Self {
        Self {
            name: name.to_owned(),
            version: version.to_owned(),
            path: path.as_ref().to_owned(),
        }
    }

    pub fn name_version_hash(&self) -> Result<String> {
        let path_str = self.path.to_str().ok_or(Error::OsStringConversion)?;
        Ok(format!("{}-{}-{}", self.name, self.version, path_str))
    }

    pub fn optional_component_paths(&self) -> OptionalComponentPaths {
        OptionalComponentPaths::new(self.binary(), self.config(), self.library(), self.share())
    }

    pub fn binary(&self) -> Option<PathBuf> {
        let binary = self.path.join("bin");
        if binary.exists() {
            Some(binary)
        } else {
            None
        }
    }

    pub fn config(&self) -> Option<PathBuf> {
        let config = self.path.join("cfg");
        if config.exists() {
            Some(config)
        } else {
            None
        }
    }

    pub fn library(&self) -> Option<PathBuf> {
        let library = self.path.join("lib");
        if library.exists() {
            Some(library)
        } else {
            None
        }
    }

    pub fn share(&self) -> Option<PathBuf> {
        let share = self.path.join("share");
        if share.exists() {
            Some(share)
        } else {
            None
        }
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}-{}", self.name, self.version))
    }
}
