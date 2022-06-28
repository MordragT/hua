use std::path::PathBuf;

use crate::{
    dependency::Requirement,
    generation::{Generation, GenerationBuilder},
    jail::{Bind, JailBuilder},
    store::{backend::ReadBackend, Store},
    HUA_PATH,
};
use os_type::OSType;
use snafu::prelude::*;
use temp_dir::TempDir;

#[derive(Debug, Snafu)]
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

#[derive(Debug, Clone)]
pub struct ShellBuilder {
    temp_dir: TempDir,
    generation: Option<Generation>,
}

impl ShellBuilder {
    pub fn new() -> ShellResult<Self> {
        Ok(Self {
            temp_dir: TempDir::new().context(IoSnafu)?,
            generation: None,
        })
    }

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
                    let blobs = unsafe { store.get_blobs_cloned_of_package(id).unwrap_unchecked() };
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

    pub fn apply(&self, jail: JailBuilder) -> ShellResult<JailBuilder> {
        let generation = self
            .generation
            .as_ref()
            .ok_or(ShellError::MissingGeneration)?;
        let gen_paths = generation.component_paths();

        // TODO bind hua store aswell so that symlinks can be found
        // only bind nix store conditonally else bind lib

        let jail = jail
            .bind(Bind::read_only("/bin", "/bin"))
            .bind(Bind::read_only(&gen_paths.binary, "/usr/bin/"))
            .bind(Bind::read_only(&gen_paths.library, "/usr/lib/"))
            .bind(Bind::read_only(&gen_paths.share, "/usr/share/"))
            .bind(Bind::read_only(&gen_paths.config, "/etc/"))
            .bind(Bind::read_only(HUA_PATH, HUA_PATH));

        let jail = match os_type::current_platform().os_type {
            OSType::NixOS => jail.bind(Bind::read_only("/nix/store", "/nix/store")),
            _ => todo!(),
        };

        Ok(jail)
    }
}
