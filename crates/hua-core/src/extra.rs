use serde::{Deserialize, Serialize};

use crate::error::*;
use std::{
    fs,
    hash::{Hash, Hasher},
    io,
    os::unix,
    path::{Path, PathBuf},
};

// TODO: better naming
// TODO io_operations should be able to return something (like when linking the linked paths)

pub fn io_operation_into<P, Q, F>(from: P, into: Q, operation: &F) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    F: Fn(&Path, &Path) -> std::io::Result<()>,
{
    let from = from.as_ref().canonicalize()?;
    let name = from
        .file_name()
        .ok_or(Error::TerminatingPath)?
        .to_str()
        .ok_or(Error::OsStringConversion)?;
    let into = into.as_ref().join(name);

    if from.is_dir() {
        from.read_dir()?
            .map(|res| match res {
                Ok(entry) => {
                    if !into.exists() {
                        fs::create_dir(&into)?;
                    }
                    io_operation_into(entry.path(), &into, operation)
                }
                Err(e) => Err(e.into()),
            })
            .collect::<Result<()>>()
    } else {
        operation(&from, &into)?;
        Ok(())
    }
}

pub fn io_operation_to<P, Q, F>(from: P, to: Q, operation: &F) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    F: Fn(&Path, &Path) -> std::io::Result<()>,
{
    let from = from.as_ref();
    let contents = from.read_dir()?;
    contents
        .map(|res| match res {
            Ok(entry) => io_operation_into(entry.path(), to.as_ref(), operation),
            Err(e) => Err(e.into()),
        })
        .collect::<Result<()>>()
}

fn symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    unix::fs::symlink(original, link)
}

/// Creates links inside the destination corresponding to the file structure of the origin
pub fn link_into<P: AsRef<Path>, Q: AsRef<Path>>(from: P, into: Q) -> Result<()> {
    io_operation_into(from, into, &symlink)
}

pub fn link_to<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    io_operation_to(from, to, &symlink)
}

/// Links one component paths into another
/// Creates the directories of the destination if they do not exist
pub fn link_component_paths(from: &ComponentPaths, to: &ComponentPaths) -> Result<()> {
    to.create_dirs()?;

    link_to(&from.binary, &to.binary)?;
    link_to(&from.config, &to.config)?;
    link_to(&from.library, &to.library)?;
    link_to(&from.share, &to.share)?;
    Ok(())
}

/// Links optional component paths to normal component paths
/// Creates the directories of the destination if they do not exist
pub fn link_optional_component_paths(
    from: &OptionalComponentPaths,
    to: &ComponentPaths,
) -> Result<()> {
    to.create_dirs()?;

    if let Some(p) = &from.binary {
        link_to(&p, &to.binary)?;
    }
    if let Some(p) = &from.config {
        link_to(&p, &to.config)?;
    }
    if let Some(p) = &from.library {
        link_to(&p, &to.library)?;
    }
    if let Some(p) = &from.share {
        link_to(&p, &to.share)?;
    }
    Ok(())
}

fn copy(from: &Path, to: &Path) -> std::io::Result<()> {
    let _num = fs::copy(from, to)?;
    Ok(())
}

/// Copies the origin inside the destination
pub fn copy_into<P: AsRef<Path>, Q: AsRef<Path>>(from: P, into: Q) -> Result<()> {
    io_operation_into(from, into, &copy)
}

/// Copies the content of the origin inside the destination
pub fn copy_to<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    io_operation_to(from, to, &copy)
}

/// Calculates a hash with all the files under the given path
pub fn hash_path<P: AsRef<Path>, H: Hasher>(path: P, state: &mut H) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        path.read_dir()?
            .map(|res| match res {
                Ok(entry) => hash_path(entry.path(), state),
                Err(e) => Err(e.into()),
            })
            .collect::<Result<()>>()
    } else {
        fs::read(path)?.hash(state);
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComponentPaths {
    pub binary: PathBuf,
    pub config: PathBuf,
    pub library: PathBuf,
    pub share: PathBuf,
}

impl ComponentPaths {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        Self {
            binary: path.join("bin"),
            config: path.join("cfg"),
            library: path.join("lib"),
            share: path.join("share"),
        }
    }

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
