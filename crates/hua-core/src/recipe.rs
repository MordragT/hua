use std::{
    collections::{BTreeSet, HashMap, HashSet},
    hash::Hash,
    path::{Path, PathBuf},
    process::Command,
};

use semver::{Version, VersionReq};

use crate::{
    error::Result, extra::Source, Component, ComponentPaths, Downloader, Error, Requirement, Store,
    UserManager,
};

/// A Recipe to build an Package from.
#[derive(Debug)]
pub struct Recipe {
    name: String,
    version: Version,
    description: String,
    archictures: u8,
    platforms: u8,
    sources: Vec<Source>,
    license: Vec<String>,
    requires: BTreeSet<Requirement>,
    requires_build: HashSet<Requirement>,
    provides: BTreeSet<Component>,
    output: Vec<PathBuf>,
    build_binaries: Option<HashMap<String, Command>>,
}

impl Hash for Recipe {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write(self.name.as_bytes());
        //state.write(self.version.as_bytes());
        self.provides.hash(state);
        self.requires.hash(state);
    }
}

// impl Dependency for Recipe {
//     fn name(&self) -> &String {
//         &self.name
//     }

//     fn version(&self) -> &String {
//         &self.version
//     }

//     fn provides(&self) -> &HashSet<Component> {
//         &self.provides
//     }

//     fn requires(&self) -> &HashSet<Box<dyn Dependency>> {
//         &self.requires
//     }
// }

impl Recipe {
    // TODO only allow packages to be a dependency ?
    /// Creates an depenency out of a recipe
    pub fn into_requirement(self, version_req: VersionReq) -> Requirement {
        Requirement::new(self.name, version_req, self.provides)
    }

    /// Creates a new Recipe.
    pub fn new(
        name: String,
        version: Version,
        description: String,
        archictures: u8,
        platforms: u8,
        sources: Vec<Source>,
        license: Vec<String>,
        requires: BTreeSet<Requirement>,
        requires_build: HashSet<Requirement>,
        provides: BTreeSet<Component>,
    ) -> Self {
        Self {
            name,
            version,
            description,
            archictures,
            platforms,
            sources,
            license,
            requires,
            requires_build,
            provides,
            output: Vec::new(),
            build_binaries: None,
        }
    }

    /// Downloads data if it is not local and verifies them.
    /// Must be called prior to unpacking even if the source is local
    pub fn download(mut self, downloader: &mut Downloader) -> Result<Self> {
        self.output = downloader.batch_download(&self.sources)?;
        Ok(self)
    }

    // TODO different files may want to be unpacked by a different unpacker

    /// Unpacks the local or downloaded data.
    pub fn unpack<T>(mut self, task: T) -> Result<Self>
    where
        T: Fn(&Path) -> Result<PathBuf>,
    {
        for source in self.output.iter_mut() {
            *source = task(&source)?;
        }

        Ok(self)
    }

    /// Link all dependencies temporarily and processes binaries
    /// for execution in the build phase.
    pub fn link_dependencies(
        mut self,
        store: &mut Store,
        global_paths: &ComponentPaths,
    ) -> Result<Self> {
        todo!()
    }

    /// Builds the recipe.
    /// In here external programs like cargo or make are run.
    pub fn build<T>(mut self, task: T) -> Result<Self>
    where
        T: FnOnce(&Path, &HashMap<String, Command>) -> Result<PathBuf>,
    {
        todo!()
    }

    /// Installs the builded recipe into the store and
    /// creates a new generation with it and its dependencies.
    /// Removes the temporary created build generation.
    /// Returns hashes of all packages installed.
    pub fn install(
        self,
        store: &mut Store,
        user_manager: &mut UserManager,
    ) -> Result<HashSet<u64>> {
        todo!()
    }
}
