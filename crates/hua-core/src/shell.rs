use crate::{
    dependency::Requirement,
    env::{Bind, Environment},
    generation::{Generation, GenerationBuilder},
    store::{backend::ReadBackend, Store, STORE_PATH},
};
use console::style;
use os_type::OSType;
use snafu::prelude::*;
use std::{fmt, path::PathBuf};
use temp_dir::TempDir;

/// An [Error](std::error::Error) of the [shell](crate::shell) module.
#[derive(Debug, Snafu)]
#[allow(missing_docs)]
pub enum ShellError {
    #[snafu(display("IoError: {source}"))]
    IoError { source: std::io::Error },
    #[snafu(display("GenerationError: {source}"))]
    GenerationError {
        source: crate::generation::GenerationError,
    },
    #[snafu(display("No package with name {name} in store"))]
    NotInStore { name: String },
    #[snafu(display("Missing Generation"))]
    MissingGeneration,
}

type ShellResult<T> = Result<T, ShellError>;

// TODO add option for shell builder for chrooted or sandboxed creation.

/// A builder for a shell.
#[derive(Debug, Clone)]
pub struct ShellBuilder {
    temp_dir: TempDir,
    generation: Option<Generation>,
}

impl ShellBuilder {
    /// Creates a new [ShellBuilder].
    pub fn new() -> ShellResult<Self> {
        Ok(Self {
            temp_dir: TempDir::new().context(IoSnafu)?,
            generation: None,
        })
    }

    /// Add [requirements](Requirement) to the shell.
    pub fn with_requirements<B: ReadBackend<Source = PathBuf>>(
        mut self,
        requirements: impl IntoIterator<Item = Requirement>,
        store: &Store<PathBuf, B>,
    ) -> ShellResult<Self> {
        let generation = GenerationBuilder::new(0)
            .under(self.temp_dir.path())
            .requires(requirements)
            .resolve(&store)
            .context(GenerationSnafu)?
            .build(&store)
            .context(GenerationSnafu)?;

        self.generation = Some(generation);
        Ok(self)
    }

    /// Try to add [requirements](Requirement) by name to the shell.
    pub fn with_names<N: AsRef<str>, B: ReadBackend<Source = PathBuf>>(
        self,
        names: impl IntoIterator<Item = N>,
        store: &Store<PathBuf, B>,
    ) -> ShellResult<Self> {
        let requirements = names
            .into_iter()
            .map(|name| {
                // TODO get the newest package not just any.
                let name = name.as_ref();
                if let Some((id, desc)) = store.packages().find_by_name_starting_with(name) {
                    let blobs = unsafe { store.blobs_cloned_of_package(id).unwrap_unchecked() };
                    Ok((desc.clone(), blobs.collect()).into())
                } else {
                    Err(ShellError::NotInStore {
                        name: name.to_owned(),
                    })
                }
            })
            .collect::<ShellResult<Vec<Requirement>>>()?;

        self.with_requirements(requirements, store)
    }

    /// Apply options to the [Environment].
    pub fn apply(&self, environment: &mut Environment) -> ShellResult<()> {
        let generation = self
            .generation
            .as_ref()
            .ok_or(ShellError::MissingGeneration)?;
        let gen_paths = generation.component_paths();

        // TODO only bind nix store conditonally else bind lib
        // TODO return error if STORE_PATH is no existing

        environment
            .bind(Bind::read_only(STORE_PATH, STORE_PATH))
            .bind(Bind::read_only("/bin", "/bin"))
            .bind(Bind::read_only(&gen_paths.binary, "/usr/bin/"))
            .bind(Bind::read_only(&gen_paths.library, "/usr/lib/"))
            .bind(Bind::read_only(&gen_paths.share, "/usr/share/"))
            .bind(Bind::read_only(&gen_paths.config, "/etc/"));

        match os_type::current_platform().os_type {
            OSType::NixOS => environment.bind(Bind::read_only("/nix/store", "/nix/store")),
            _ => todo!(),
        };

        Ok(())
    }
}

impl fmt::Display for ShellBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.generation {
            Some(gen) => write!(f, "{}:\n{gen}", style("Shell").green()),
            None => write!(f, "{}", style("Empty Shell").red()),
        }
    }
}
