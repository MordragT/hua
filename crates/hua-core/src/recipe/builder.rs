use super::*;
use crate::{
    dependency::Requirement,
    recipe::Recipe,
    store::{backend::ReadBackend, package::PackageSource, LocalStore, Store},
};
use cached_path::Cache;
use relative_path::RelativePathBuf;
use semver::Version;
use serde::Deserialize;
use std::{collections::HashSet, path::PathBuf};

#[derive(Debug, Deserialize)]
pub struct RecipeData {
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

pub fn build_recipe(
    data: RecipeData,
    store: &mut LocalStore,
    cache: &Cache,
) -> RecipeResult<Option<PathBuf>> {
    let RecipeData {
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
    } = data;

    let recipe = Recipe::new(
        name,
        version,
        desc,
        archs,
        platforms,
        source,
        licenses,
        requires,
        requires_build,
        target_dir,
    );

    recipe
        .fetch(&cache)?
        .prepare_requirements(&store, vars.into_iter())?
        .build(script)?
        .install(store)
}
