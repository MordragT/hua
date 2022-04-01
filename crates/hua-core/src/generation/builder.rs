use super::*;
use crate::extra::path::ComponentPathBuf;
use crate::store::{Backend, PackageId};
use crate::Store;
use crate::{DependencyGraph, Requirement};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

// TODO: Tests

#[derive(Debug)]
pub struct GenerationBuilder {
    id: usize,
    requirements: Option<HashSet<Requirement>>,
    packages: Option<HashSet<PackageId>>,
    base: Option<PathBuf>,
}

impl GenerationBuilder {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            requirements: None,
            packages: None,
            base: None,
        }
    }

    pub fn under<P: AsRef<Path>>(mut self, base: P) -> Self {
        self.base = Some(base.as_ref().to_owned());
        self
    }

    pub fn requires(mut self, requirements: impl IntoIterator<Item = Requirement>) -> Self {
        self.requirements = Some(requirements.into_iter().collect());
        self
    }

    pub fn resolve<B: Backend>(mut self, store: &Store<B>) -> GenerationResult<Self> {
        if let Some(reqs) = &self.requirements {
            let mut graph = DependencyGraph::new();
            let packages = graph
                .resolve(reqs, store)
                .context(DependencySnafu { id: self.id })?
                .collect();

            self.packages = Some(packages);
            Ok(self)
        } else {
            Err(GenerationError::MissingRequirements { id: self.id })
        }
    }

    pub fn build<B: Backend>(self, store: &Store<B>) -> GenerationResult<Generation> {
        let base = self
            .base
            .ok_or(GenerationError::MissingBasePath { id: self.id })?;

        let requirements = self
            .requirements
            .ok_or(GenerationError::MissingRequirements { id: self.id })?;

        let packages = self
            .packages
            .ok_or(GenerationError::MissingPackages { id: self.id })?;

        let path = base.join(self.id.to_string());
        if path.exists() {
            return Err(GenerationError::AlreadyPresent { id: self.id });
        }
        fs::create_dir(&path).context(GenerationIoSnafu { id: self.id })?;
        let component_paths = ComponentPathBuf::new(
            path.join("bin"),
            path.join("cfg"),
            path.join("lib"),
            path.join("share"),
        );
        component_paths
            .create_dirs()
            .context(GenerationIoSnafu { id: self.id })?;

        store
            .link_packages(&packages, &component_paths)
            .context(StoreSnafu)?;

        Ok(Generation::new(
            path,
            packages,
            requirements,
            component_paths,
        ))
    }

    pub fn empty(self) -> GenerationResult<Generation> {
        let base = self
            .base
            .ok_or(GenerationError::MissingBasePath { id: self.id })?;

        let path = base.join(self.id.to_string());
        if path.exists() {
            return Err(GenerationError::AlreadyPresent { id: self.id });
        }
        fs::create_dir(&path).context(GenerationIoSnafu { id: self.id })?;
        let component_paths = ComponentPathBuf::new(
            path.join("bin"),
            path.join("cfg"),
            path.join("lib"),
            path.join("share"),
        );
        component_paths
            .create_dirs()
            .context(GenerationIoSnafu { id: self.id })?;

        Ok(Generation::new(
            path,
            HashSet::new(),
            HashSet::new(),
            component_paths,
        ))
    }
}
