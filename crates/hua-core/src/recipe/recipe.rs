use super::*;
use crate::{
    dependency::Requirement,
    extra::hash::PackageHash,
    generation::GenerationBuilder,
    jail::{Bind, JailBuilder},
    store::{Backend, Package, PackageDesc, Store, STORE_PATH},
};
use cached_path::{Cache, Options};
use fs_extra::dir::CopyOptions;
use os_type::OSType;
use relative_path::RelativePathBuf;
use semver::Version;
use std::{
    collections::HashSet,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use temp_dir::TempDir;

// TODO: implement patches, allow multiple sources ?

pub const BUILD_PATH: &str = "/tmp/build/";
pub const BUILD_SCRIPT_PATH: &str = "/tmp/build.sh";

/// A Recipe to build an Package from.
#[derive(Debug)]
pub struct Recipe {
    name: String,
    version: Version,
    desc: String,
    archs: u8,
    platforms: u8,
    source: String,
    licenses: Vec<String>,
    requires: HashSet<Requirement>,
    requires_build: HashSet<Requirement>,
    target_dir: RelativePathBuf,
    jail: Option<JailBuilder>,
    build_generation_dir: Option<TempDir>,
    build_dir: Option<PathBuf>,
}

impl Recipe {
    /// Creates a new Recipe.
    pub fn new(
        name: String,
        version: Version,
        desc: String,
        archictures: u8,
        platforms: u8,
        source: String,
        licenses: Vec<String>,
        requires: HashSet<Requirement>,
        requires_build: HashSet<Requirement>,
        target_dir: RelativePathBuf,
    ) -> Self {
        Self {
            name,
            version,
            desc,
            archs: archictures,
            platforms,
            source,
            licenses,
            requires,
            requires_build,
            target_dir,
            jail: None,
            build_generation_dir: None,
            build_dir: None,
        }
    }

    // TODO verify sources when downloaded

    /// Downloads data if it is not local and verifies them.
    /// Must be called prior to unpacking even if the source is local
    pub fn fetch(mut self, cache: &Cache) -> RecipeResult<Self> {
        super::check_archs(self.archs)?;
        super::check_platforms(self.platforms)?;

        let path = cache
            .cached_path_with_options(&self.source, &Options::default().extract())
            .context(CacheSnafu)?;

        let build_dir = PathBuf::from("build");
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).context(IoSnafu)?;
        }

        let mut copy_options = CopyOptions::default();
        copy_options.copy_inside = true;
        copy_options.skip_exist = true;
        fs_extra::dir::copy(path, &build_dir, &copy_options).context(FsExtraSnafu)?;

        self.build_dir = Some(build_dir);
        Ok(self)
    }

    // TODO JailBuilder as argument, so that users can provide envs or envs in recipe itself

    /// Link all dependencies temporarily and processes binaries
    /// for execution in the build phase.
    pub fn prepare_requirements<B: Backend, L: AsRef<str>, R: AsRef<str>>(
        mut self,
        store: &Store<B>,
        vars: impl IntoIterator<Item = (L, R)>,
    ) -> RecipeResult<Self> {
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
            //.bind(Bind::read_only(STORE_PATH, STORE_PATH))
            .bind(Bind::read_only(&tmp_gen_paths.binary, "/usr/bin/"))
            .bind(Bind::read_only(&tmp_gen_paths.library, "/usr/lib/"))
            .bind(Bind::read_only(&tmp_gen_paths.share, "/usr/share/"))
            .bind(Bind::read_only(&tmp_gen_paths.config, "/etc/"))
            .bind(Bind::read_write(build_dir, BUILD_PATH))
            .envs(vars)
            .current_dir(BUILD_PATH);

        let jail = match os_type::current_platform().os_type {
            OSType::NixOS => jail.bind(Bind::read_only("/nix/store", "/nix/store")),
            _ => todo!(),
        };

        self.build_generation_dir = Some(build_generation_dir);
        self.jail = Some(jail);
        Ok(self)
    }

    /// Builds the recipe.
    /// In here external programs like cargo or make are run.
    pub fn build(self, script: String) -> RecipeResult<(Package, PathBuf)> {
        let jail = self.jail.ok_or(RecipeError::MissingJail)?;
        let build_dir = self.build_dir.ok_or(RecipeError::MissingSourceFiles)?;

        let temp_dir = TempDir::new().context(IoSnafu)?;
        let script_path = temp_dir.child(&self.name);
        let mut script_file = File::create(&script_path).context(IoSnafu)?;
        script_file.write(script.as_bytes()).context(IoSnafu)?;

        let mut process = jail
            // .file(script_file.as_raw_fd(), BUILD_SCRIPT_PATH, 544)
            // .arg("sh")
            // .arg(BUILD_SCRIPT_PATH)
            .bind(Bind::read_only(script_path, BUILD_SCRIPT_PATH))
            .arg("sh")
            .arg(BUILD_SCRIPT_PATH)
            .run()
            .context(IoSnafu)?;
        let ecode = process.wait().context(IoSnafu)?;
        assert!(ecode.success());

        let absolute_target_dir = self
            .target_dir
            .to_path(build_dir.canonicalize().context(IoSnafu)?);

        let PackageHash {
            root,
            trees: _,
            blobs: _,
        } = PackageHash::from_path(&absolute_target_dir, &self.name).context(IoSnafu)?;

        let desc = PackageDesc::new(
            self.name,
            self.desc,
            self.version,
            self.licenses,
            self.requires,
        );
        let package = Package::new(root, desc);

        let toml = toml::to_vec(&package).context(TomlSerilizationSnafu)?;

        let lock_path = PathBuf::from(format!("{}.lock", package.name()));
        if lock_path.exists() {
            fs::remove_file(&lock_path).context(IoSnafu)?;
        }
        let mut lock = File::create(lock_path).context(IoSnafu)?;
        lock.write_all(&toml).context(IoSnafu)?;

        Ok((package, absolute_target_dir))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs};

    use cached_path::CacheBuilder;
    use relative_path::RelativePathBuf;
    use semver::Version;
    use temp_dir::TempDir;

    use crate::{
        recipe::{Recipe, LINUX, X86, X86_64},
        store::{LocalBackend, Store},
    };

    #[test]
    fn recipe_automake() {
        let name = "automake".to_owned();
        let version = Version::new(1, 16, 4);
        let description = String::new();
        let archictures = X86_64 | X86;
        let platforms = LINUX;
        // TODO checksum
        let source = format!("https://ftp.gnu.org/gnu/automake/automake-{version}.tar.gz");
        let license = vec!["GPLv2".to_owned()];
        let requires = HashSet::new();
        let requires_build = HashSet::new();
        let target_dir = RelativePathBuf::from_path(format!("{name}-{version}")).unwrap();

        let recipe = Recipe::new(
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
        );

        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");
        let mut store = Store::<LocalBackend>::init(store_path).unwrap();
        let cache = CacheBuilder::new().build().unwrap();

        let recipe_path = temp_dir.child("recipe");
        fs::create_dir(&recipe_path).unwrap();
        std::env::set_current_dir(recipe_path).unwrap();

        let (package, path) = recipe
            .fetch(&cache)
            .unwrap()
            .prepare_requirements::<LocalBackend, &str, &str>(&store, [])
            .unwrap()
            .build("ls".to_owned())
            .unwrap();

        assert!(store.insert(package, path).unwrap());
    }
}
