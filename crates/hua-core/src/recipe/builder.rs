use super::*;
use crate::{
    dependency::Requirement,
    recipe::Recipe,
    store::{id::PackageId, LocalStore},
};
use cached_path::Cache;
use console::style;
use relative_path::RelativePathBuf;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Derivation {
    pub name: String,
    pub version: Version,
    pub desc: String,
    pub archs: u8,
    pub platforms: u8,
    pub source: String,
    pub licenses: Vec<String>,
    pub requires: HashSet<Requirement>,
    pub requires_build: HashSet<Requirement>,
    pub vars: Vec<(String, String)>,
    pub script: String,
    pub target_dir: RelativePathBuf,
}

impl Derivation {
    pub fn new(
        name: String,
        version: Version,
        desc: String,
        archs: u8,
        platforms: u8,
        source: String,
        licenses: Vec<String>,
        requires: HashSet<Requirement>,
        requires_build: HashSet<Requirement>,
        vars: Vec<(String, String)>,
        script: String,
        target_dir: RelativePathBuf,
    ) -> Self {
        Self {
            name,
            version,
            desc,
            archs,
            platforms,
            source,
            licenses,
            requires,
            requires_build,
            vars,
            script,
            target_dir,
        }
    }

    pub fn path_in_store<P: AsRef<Path>>(&self, store_path: P, id: &PackageId) -> PathBuf {
        let relative_path = self.relative_path(id);
        relative_path.to_path(store_path.as_ref())
    }

    pub fn relative_path(&self, id: &PackageId) -> RelativePathBuf {
        RelativePathBuf::from(format!("{}-{}-{}", self.name, self.version, id))
    }

    // pub fn url_in_store(&self, store_url: &Url, id: &PackageId) -> Url {
    //     let name_version_id = format!("{}-{}-{}", self.name, self.version, id);
    //     store_url
    //         .join(&name_version_id)
    //         .expect("Should always be valid")
    // }
}

impl fmt::Display for Derivation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Derivation {}\n", style(&self.name).blue())?;
        write!(f, "Version {}\n", style(&self.version).blue())?;
        write!(f, "Description {}\n", style(&self.desc).blue())?;
        write!(f, "Source {}\n", style(&self.source).blue())?;
        write!(f, "Licenses: \n")?;
        for license in &self.licenses {
            write!(f, "\t{}\n", style(license).blue())?;
        }
        write!(f, "Requires: \n")?;
        for req in &self.requires {
            write!(f, "\t{}\n", req)?;
        }
        write!(f, "Requires in build: \n")?;
        for req in &self.requires_build {
            write!(f, "\t{}\n", req)?;
        }
        write!(f, "Environmental variables: \n")?;
        for var in &self.vars {
            write!(f, "{} = {}", style(&var.0).blue(), style(&var.1).red())?;
        }
        write!(f, "Script: {}\n", self.script)?;
        write!(f, "Target directory {}\n", style(&self.target_dir).blue())?;
        Ok(())
    }
}

pub fn build_recipe(
    drv: Derivation,
    store: &mut LocalStore,
    cache: &Cache,
) -> RecipeResult<PathBuf> {
    let recipe = Recipe::new(drv);

    recipe
        .fetch(&cache)?
        .prepare_requirements(&store)?
        .build()?
        .install(store)
}
