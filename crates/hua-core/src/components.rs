use std::{
    collections::hash_set,
    collections::HashSet,
    fs,
    hash::Hash,
    io,
    iter::{Chain, Map},
    path::{Path, PathBuf},
};

use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

use crate::{error::Result, Error};

#[derive(PartialEq, Eq, Debug, Hash, Serialize, Deserialize, Clone, PartialOrd, Ord)]
pub enum Binary {
    Shell(RelativePathBuf),
    Elf(RelativePathBuf),
}

impl Binary {
    /// Return the inner relative path
    pub fn relative_path(&self) -> &RelativePath {
        match self {
            Self::Shell(path) => &path,
            Self::Elf(path) => &path,
        }
    }
}

impl Into<RelativePathBuf> for Binary {
    fn into(self) -> RelativePathBuf {
        match self {
            Self::Shell(path) => path,
            Self::Elf(path) => path,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash, PartialOrd, Ord)]
pub enum Component {
    Binary(Binary),
    Config(RelativePathBuf),
    Library(RelativePathBuf),
    Share(RelativePathBuf),
}

impl Component {
    /// Gets the underlying relative path
    pub fn relative_path(&self) -> &RelativePath {
        match &self {
            Self::Binary(b) => b.relative_path(),
            Self::Config(p) | Self::Library(p) | Self::Share(p) => &p,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Components {
    binaries: HashSet<Binary>,
    configs: HashSet<RelativePathBuf>,
    libraries: HashSet<RelativePathBuf>,
    shares: HashSet<RelativePathBuf>,
}

impl Components {
    pub fn is_disjoint(&self, other: &Components) -> bool {
        self.binaries.is_disjoint(&other.binaries)
            && self.configs.is_disjoint(&other.configs)
            && self.libraries.is_disjoint(&other.libraries)
            && self.shares.is_disjoint(&other.shares)
    }

    pub fn binaries(&self) -> &HashSet<Binary> {
        &self.binaries
    }

    pub fn configs(&self) -> &HashSet<RelativePathBuf> {
        &self.configs
    }

    pub fn libraries(&self) -> &HashSet<RelativePathBuf> {
        &self.libraries
    }

    pub fn shares(&self) -> &HashSet<RelativePathBuf> {
        &self.shares
    }

    pub fn new_binary(binary: &str) -> Result<Self> {
        let relative_path = RelativePathBuf::from(binary);
        let mut binaries = HashSet::new();

        if !relative_path.starts_with("bin") {
            return Err(Error::WrongPathParent(relative_path));
        } else if let Some(ext) = relative_path.extension() && ext == "sh" {
            binaries.insert(Binary::Shell(relative_path));
        } else {
            binaries.insert(Binary::Elf(relative_path));
        }

        Ok(Self {
            binaries,
            configs: HashSet::new(),
            libraries: HashSet::new(),
            shares: HashSet::new(),
        })
    }
}

impl IntoIterator for Components {
    type Item = RelativePathBuf;
    type IntoIter = Chain<
        Map<hash_set::IntoIter<Binary>, fn(Binary) -> RelativePathBuf>,
        Chain<
            hash_set::IntoIter<RelativePathBuf>,
            Chain<hash_set::IntoIter<RelativePathBuf>, hash_set::IntoIter<RelativePathBuf>>,
        >,
    >;

    fn into_iter(self) -> Self::IntoIter {
        let res = self
            .binaries
            .into_iter()
            .map(Binary::into as fn(Binary) -> RelativePathBuf)
            .chain(
                self.configs
                    .into_iter()
                    .chain(self.libraries.into_iter().chain(self.shares.into_iter())),
            );
        res
    }
}

impl Hash for Components {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for binary in &self.binaries {
            binary.hash(state);
        }

        for config in &self.configs {
            config.hash(state);
        }

        for library in &self.libraries {
            library.hash(state);
        }

        for share in &self.shares {
            share.hash(state);
        }
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
