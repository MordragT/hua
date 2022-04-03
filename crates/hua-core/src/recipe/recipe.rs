use super::*;
use crate::{
    extra::hash::PackageHash,
    jail::{Bind, Jail, JailBuilder},
    store::Backend,
    GenerationBuilder, Package, Requirement, Store,
};
use cached_path::{Cache, Options};
use relative_path::RelativePathBuf;
use semver::Version;
use std::{collections::HashSet, path::PathBuf};
use temp_dir::TempDir;

// TODO: implement patches, allow multiple sources ?

/// A Recipe to build an Package from.
#[derive(Debug)]
pub struct Recipe<'a> {
    name: String,
    version: Version,
    description: String,
    archictures: u8,
    platforms: u8,
    source: &'a str,
    license: Vec<String>,
    requires: HashSet<Requirement>,
    requires_build: HashSet<Requirement>,
    target_dir: RelativePathBuf,
    // output: Option<Vec<PathBuf>>,
    jail: Option<Jail<'a>>,
    build_generation_dir: Option<TempDir>,
    build_dir: Option<PathBuf>,
}

impl<'a> Recipe<'a> {
    /// Creates a new Recipe.
    pub fn new(
        name: String,
        version: Version,
        description: String,
        archictures: u8,
        platforms: u8,
        source: &'a str,
        license: Vec<String>,
        requires: HashSet<Requirement>,
        requires_build: HashSet<Requirement>,
        target_dir: RelativePathBuf,
    ) -> Self {
        Self {
            name,
            version,
            description,
            archictures,
            platforms,
            source,
            license,
            requires,
            requires_build,
            target_dir,
            // output: None,
            jail: None,
            build_generation_dir: None,
            build_dir: None,
        }
    }

    // TODO verify sources when downloaded

    /// Downloads data if it is not local and verifies them.
    /// Must be called prior to unpacking even if the source is local
    pub fn fetch(mut self, cache: &Cache) -> RecipeResult<Self> {
        if cfg!(target_arch = "x86_64") && self.archictures & X86_64 != X86_64 {
            return Err(RecipeError::IncompatibleArchitecture);
        }
        if cfg!(target_os = "linux") && self.platforms & LINUX != LINUX {
            return Err(RecipeError::IncompatiblePlatform);
        }

        let path = cache
            .cached_path_with_options(&self.source, &Options::default().extract())
            .context(CacheSnafu)?;

        self.build_dir = Some(path);
        Ok(self)
    }

    // TODO JailBuilder as argument, so that users can provide envs or envs in recipe itself

    /// Link all dependencies temporarily and processes binaries
    /// for execution in the build phase.
    pub fn prepare_requirements<B: Backend>(mut self, store: &Store<B>) -> RecipeResult<Self> {
        let build_dir = self
            .build_dir
            .as_ref()
            .ok_or(RecipeError::MissingSourceFiles)?;

        let requirements = self
            .requires
            .clone()
            .into_iter()
            .chain(self.requires_build.clone().into_iter());

        let build_generation_dir = TempDir::new().context(IoSnafu)?;
        let tmp_gen = GenerationBuilder::new(0)
            .under(build_generation_dir.path())
            .requires(requirements)
            .resolve(store)
            .context(GenerationSnafu)?
            .build(store)
            .context(GenerationSnafu)?;
        let tmp_gen_paths = tmp_gen.component_paths();

        // TODO bind hua store aswell so that symlinks can be found
        // only bind nix store conditonally else bind lib

        let jail = JailBuilder::new()
            .bind(Bind::read_only("/bin", "/bin"))
            .bind(Bind::read_only("/nix/store", "/nix/store"))
            .bind(Bind::read_only(&tmp_gen_paths.binary, "/usr/bin/"))
            .bind(Bind::read_only(&tmp_gen_paths.library, "/usr/lib/"))
            .bind(Bind::read_only(&tmp_gen_paths.share, "/usr/share/"))
            .bind(Bind::read_only(&tmp_gen_paths.config, "/etc/"))
            .bind(Bind::read_write(build_dir, "/tmp/build/"))
            .current_dir("/tmp/build/")
            .build();

        self.build_generation_dir = Some(build_generation_dir);
        self.jail = Some(jail);
        Ok(self)
    }

    /// Builds the recipe.
    /// In here external programs like cargo or make are run.
    pub fn build(
        mut self,
        args: impl IntoIterator<Item = &'a str>,
    ) -> RecipeResult<(Package, PathBuf)> {
        let jail = self.jail.as_mut().ok_or(RecipeError::MissingJail)?;
        let build_dir = self.build_dir.ok_or(RecipeError::MissingSourceFiles)?;

        let mut process = jail.args(args).run().context(IoSnafu)?;
        let ecode = process.wait().context(IoSnafu)?;
        assert!(ecode.success());

        let absolute_target_dir = self.target_dir.to_path(build_dir);

        let PackageHash { root, trees, blobs } =
            PackageHash::from_path(&absolute_target_dir, &self.name).context(IoSnafu)?;

        let package = Package::new(root, self.name, self.version, trees, blobs, self.requires);
        Ok((package, absolute_target_dir))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use cached_path::CacheBuilder;
    use relative_path::RelativePathBuf;
    use semver::Version;
    use temp_dir::TempDir;

    use crate::{
        recipe::{LINUX, X86_64},
        store::LocalBackend,
        Recipe, Store,
    };

    #[test]
    fn recipe_automake() {
        let name = "automake";
        let version = Version::new(1, 16, 4);
        let description = String::new();
        let archictures = X86_64;
        let platforms = LINUX;
        let source = format!("https://ftp.gnu.org/gnu/automake/automake-{version}.tar.gz");
        let license = vec!["GPLv2".to_owned()];
        let requires = HashSet::new();
        let requires_build = HashSet::new();
        let target_dir = RelativePathBuf::from_path(format!("{name}-{version}")).unwrap();

        let recipe = Recipe::new(
            name.to_owned(),
            version,
            description,
            archictures,
            platforms,
            &source,
            license,
            requires,
            requires_build,
            target_dir,
        );

        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");
        let store = Store::<LocalBackend>::init(store_path).unwrap();
        let cache = CacheBuilder::new().build().unwrap();

        let (package, path) = recipe
            .fetch(&cache)
            .unwrap()
            .prepare_requirements(&store)
            .unwrap()
            .build(["sh", "-c", "ls"])
            .unwrap();

        dbg!(package);
        dbg!(path);
    }
}
