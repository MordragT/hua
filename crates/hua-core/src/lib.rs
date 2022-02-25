mod error;
mod extra;
mod generation;
mod package;
mod store;
mod user;

use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub use error::Error;
pub use generation::*;
pub use package::Package;
pub use store::Store;
pub use user::*;

#[derive(Debug)]
pub struct ComponentPaths {
    pub binary: PathBuf,
    pub config: PathBuf,
    pub library: PathBuf,
    pub share: PathBuf,
}

impl ComponentPaths {
    pub fn new<T, U, V, W>(binary: T, config: U, library: V, share: W) -> Self
    where
        T: AsRef<Path>,
        U: AsRef<Path>,
        V: AsRef<Path>,
        W: AsRef<Path>,
    {
        Self {
            binary: binary.as_ref().to_owned(),
            config: config.as_ref().to_owned(),
            library: library.as_ref().to_owned(),
            share: share.as_ref().to_owned(),
        }
    }

    pub fn create_dirs(&self) -> io::Result<()> {
        if !self.binary.exists() {
            fs::create_dir(&self.binary)?;
        }
        if !self.config.exists() {
            fs::create_dir(&self.config)?;
        }
        if !self.library.exists() {
            fs::create_dir(&self.library)?;
        }
        if !self.share.exists() {
            fs::create_dir(&self.share)?;
        }
        Ok(())
    }
}
