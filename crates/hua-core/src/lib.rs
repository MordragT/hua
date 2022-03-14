#![feature(let_chains)]

mod error;
mod extra;
mod generation;
mod package;
mod persist;
mod store;
mod user;

use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub use error::Error;
pub use generation::*;
pub use package::Package;
use serde::{Deserialize, Serialize};
pub use store::Store;
pub use user::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OptionalComponentPaths {
    pub binary: Option<PathBuf>,
    pub config: Option<PathBuf>,
    pub library: Option<PathBuf>,
    pub share: Option<PathBuf>,
}

impl OptionalComponentPaths {
    pub fn new<T, U, V, W>(
        binary: Option<T>,
        config: Option<U>,
        library: Option<V>,
        share: Option<W>,
    ) -> Self
    where
        T: AsRef<Path>,
        U: AsRef<Path>,
        V: AsRef<Path>,
        W: AsRef<Path>,
    {
        Self {
            binary: binary.map(|p| p.as_ref().to_owned()),
            config: config.map(|p| p.as_ref().to_owned()),
            library: library.map(|p| p.as_ref().to_owned()),
            share: share.map(|p| p.as_ref().to_owned()),
        }
    }

    pub fn create_dirs(&self) -> io::Result<()> {
        if let Some(p) = &self.binary && !p.exists() {
            fs::create_dir(&p)?;
        }
        if let Some(p) = &self.config && !p.exists() {
            fs::create_dir(&p)?;
        }
        if let Some(p) = &self.library && !p.exists() {
            fs::create_dir(&p)?;
        }
        if let Some(p) = &self.share && !p.exists() {
            fs::create_dir(&p)?;
        }
        Ok(())
    }
}
