use super::*;
use crate::{
    jail::{Bind, JailBuilder},
    shell::ShellBuilder,
    store::{backend::ReadBackend, package::LocalPackageSource, LocalStore, Store},
};
use cached_path::{Cache, Options};
use fs_extra::dir::CopyOptions;
use log::{debug, info};
use snafu::ResultExt;
use std::{fs::File, io::Write, os::unix, path::PathBuf};
use temp_dir::TempDir;

// TODO: implement patches, allow multiple sources ?

const BUILD_PATH: &str = "/tmp/build/";
const BUILD_SCRIPT_PATH: &str = "/tmp/build.sh";

/// A Recipe to build an Package from.
#[derive(Debug)]
pub struct Recipe {
    drv: Derivation,
    jail: Option<JailBuilder>,
    shell: Option<ShellBuilder>,
    build_dir: Option<PathBuf>,
    temp_dir: Option<TempDir>,
    absolute_target_dir: Option<PathBuf>,
}

impl From<Derivation> for Recipe {
    fn from(drv: Derivation) -> Self {
        Self::new(drv)
    }
}

impl Recipe {
    /// Creates a new Recipe.
    pub fn new(drv: Derivation) -> Self {
        Self {
            drv,
            jail: None,
            shell: None,
            build_dir: None,
            temp_dir: None,
            absolute_target_dir: None,
        }
    }

    // TODO verify sources when downloaded

    /// Downloads data if it is not local and verifies them.
    /// Must be called prior to unpacking even if the source is local
    pub fn fetch(mut self, cache: &Cache) -> RecipeResult<Self> {
        super::check_archs(self.drv.archs)?;
        super::check_platforms(self.drv.platforms)?;

        info!("Checked architecture and platform");

        // let lock_path = PathBuf::from(format!("{}.lock", self.drv.name));
        // if lock_path.exists() {
        //     return Err(RecipeError::LockFileExists { path: lock_path });
        // }
        let link = PathBuf::from("result");
        if link.exists() {
            return Err(RecipeError::ResultLinkExists);
        }

        info!("Checked result link");

        let path = cache
            .cached_path_with_options(&self.drv.source, &Options::default().extract())
            .context(CacheSnafu)?;

        info!("Downloaded source");

        let temp_dir = TempDir::new().context(IoSnafu)?;

        let build_dir = temp_dir.child("build");

        info!("Build directory created");

        let mut copy_options = CopyOptions::default();
        copy_options.copy_inside = true;
        copy_options.skip_exist = true;
        fs_extra::dir::copy(dbg!(path), &build_dir, &copy_options).context(FsExtraSnafu)?;

        info!("Copied downloaded data into build directory");

        self.build_dir = Some(build_dir);
        self.temp_dir = Some(temp_dir);
        Ok(self)
    }

    /// Link all dependencies temporarily and processes binaries
    /// for execution in the build phase.
    pub fn prepare_requirements<B: ReadBackend<Source = PathBuf>>(
        mut self,
        store: &Store<PathBuf, B>,
    ) -> RecipeResult<Self> {
        let build_dir = self
            .build_dir
            .as_ref()
            .ok_or(RecipeError::MissingSourceFiles)?;

        let requirements = self
            .drv
            .requires
            .clone()
            .into_iter()
            .chain(self.drv.requires_build.clone().into_iter());

        let jail = JailBuilder::new()
            .bind(Bind::read_write(&build_dir, BUILD_PATH))
            .envs(self.drv.vars.clone())
            .current_dir(BUILD_PATH);

        info!("Created jail");

        let shell = ShellBuilder::new()
            .context(ShellSnafu)?
            .with_requirements(requirements, &store)
            .context(ShellSnafu)?;

        info!("Created shell env");

        let jail = shell.apply(jail).context(ShellSnafu)?;

        info!("Applied shell env to jail");

        self.shell = Some(shell);
        self.jail = Some(jail);
        Ok(self)
    }

    /// Builds the recipe.
    /// In here external programs like cargo or make are run.
    pub fn build(mut self) -> RecipeResult<Self> {
        let jail = self.jail.ok_or(RecipeError::MissingJail)?;
        let build_dir = self.build_dir.ok_or(RecipeError::MissingSourceFiles)?;
        let temp_dir = self.temp_dir.as_ref().ok_or(RecipeError::MissingTempDir)?;

        let script_path = temp_dir.child(&self.drv.name);
        let mut script_file = File::create(&script_path).context(IoSnafu)?;
        script_file
            .write(self.drv.script.as_bytes())
            .context(IoSnafu)?;

        temp_dir.leak();

        debug!("Written temprorary script file {script_path:?}");

        let mut process = jail
            .bind(Bind::read_only(script_path, BUILD_SCRIPT_PATH))
            .arg("sh")
            .arg(BUILD_SCRIPT_PATH)
            .run()
            .context(IoSnafu)?;
        let ecode = process.wait().context(IoSnafu)?;
        assert!(ecode.success());

        info!("Completed build script");

        let absolute_target_dir = self.drv.target_dir.to_path(build_dir);

        debug!("Calculated absolute dir {absolute_target_dir:?}");

        self.build_dir = None;
        self.jail = None;
        self.absolute_target_dir = Some(absolute_target_dir);
        Ok(self)
    }

    pub fn install(self, store: &mut LocalStore) -> RecipeResult<PathBuf> {
        let absolute_target_dir = self
            .absolute_target_dir
            .ok_or(RecipeError::MissingTargetDir)?;

        let package_source = LocalPackageSource::new(self.drv, absolute_target_dir);

        let path = store.insert(package_source).context(StoreSnafu)?;
        let link = PathBuf::from("result");
        unix::fs::symlink(&path, &link).context(IoSnafu)?;

        info!("Created result link");
        Ok(link)
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
        recipe::{Derivation, Recipe, LINUX, X86, X86_64},
        store::{backend::LocalBackend, LocalStore},
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

        let recipe: Recipe = Derivation::new(
            name,
            version,
            description,
            archictures,
            platforms,
            source,
            license,
            requires,
            requires_build,
            Vec::new(),
            "ls".to_owned(),
            target_dir,
        )
        .into();

        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");
        let mut store = LocalStore::init(store_path).unwrap();
        let cache = CacheBuilder::new().build().unwrap();

        let recipe_path = temp_dir.child("recipe");
        fs::create_dir(&recipe_path).unwrap();
        std::env::set_current_dir(recipe_path).unwrap();

        recipe
            .fetch(&cache)
            .unwrap()
            .prepare_requirements::<LocalBackend>(&store)
            .unwrap()
            .build()
            .unwrap()
            .install(&mut store)
            .unwrap();
    }
}
