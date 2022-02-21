use crate::{error::*, store::Store};
use std::collections::HashMap;
use std::ops::RangeBounds;
use std::os::unix;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Generation {
    path: PathBuf,
}

/// Creates links inside the destination corresponding to the file structure of the origin
fn link_children<P: AsRef<Path>, Q: AsRef<Path>>(origin: P, destination: Q) -> Result<()> {
    let origin = origin.as_ref();
    let name = origin
        .file_name()
        .ok_or(Error::TerminatingPath)?
        .to_str()
        .ok_or(Error::OsStringConversion)?;
    let destination = destination.as_ref().join(name);

    if origin.is_dir() {
        origin
            .read_dir()?
            .map(|res| match res {
                Ok(entry) => {
                    if !destination.exists() {
                        fs::create_dir(&destination)?;
                    }
                    link_children(entry.path(), &destination)
                }
                Err(e) => Err(e.into()),
            })
            .collect::<Result<()>>()
    } else {
        unix::fs::symlink(&origin, &destination)?;
        Ok(())
    }
}

impl Generation {
    pub fn create_under<P: AsRef<Path>>(path: P, id: usize) -> Result<Self> {
        let path = path.as_ref().join(id.to_string());
        fs::create_dir(&path)?;
        Ok(Self { path })
    }

    pub fn link_package(&mut self, hash: u64, store: &Store) -> Result<()> {
        let package = store.get(hash).ok_or(Error::PackageNotFound(hash))?;
        if let Some(binary) = package.binary() {
            link_children(binary, &self.path)?;
        }
        if let Some(config) = package.config() {
            link_children(config, &self.path)?;
        }
        if let Some(library) = package.library() {
            link_children(library, &self.path)?;
        }
        if let Some(share) = package.share() {
            link_children(share, &self.path)?;
        }
        Ok(())
    }

    pub fn binary(&self) -> Result<PathBuf> {
        let binary = self.path.join("/bin");
        if !binary.exists() {
            fs::create_dir(&binary)?;
        }
        Ok(binary)
    }

    pub fn config(&self) -> Result<PathBuf> {
        let config = self.path.join("/cfg");
        if !config.exists() {
            fs::create_dir(&config)?;
        }
        Ok(config)
    }

    pub fn library(&self) -> Result<PathBuf> {
        let library = self.path.join("/lib");
        if !library.exists() {
            fs::create_dir(&library)?;
        }
        Ok(library)
    }

    pub fn share(&self) -> Result<PathBuf> {
        let share = self.path.join("/share");
        if !share.exists() {
            fs::create_dir(&share)?;
        }
        Ok(share)
    }
}

#[derive(Debug)]
pub struct GenerationManager {
    path: PathBuf,
    current: usize,
    next: Option<usize>,
    list: HashMap<usize, Generation>,
}

impl GenerationManager {
    pub fn create_under<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().join("/generations");
        fs::create_dir(&path)?;
        let current = 0;
        let mut list = HashMap::new();
        list.insert(current, Generation::create_under(&path, current)?);

        Ok(Self {
            path,
            current,
            next: Option::None,
            list,
        })
    }

    pub fn get_current(&self) -> &Generation {
        self.list.get(&self.current).unwrap()
    }

    pub fn switch_to(&mut self, id: usize) -> Result<()> {
        if self.list.contains_key(&id) {
            self.current = id;
            // TODO switch symlinks
            Ok(())
        } else {
            Err(Error::GenerationNotExistent(id))
        }
    }
}
