#![feature(let_chains)]
#![feature(slice_pattern)]
#![feature(assert_matches)]
#![feature(explicit_generic_args_with_impl_trait)]
#![feature(c_size_t)]
#![feature(unix_chown)]

pub mod c_ffi;
pub mod dependency;
pub mod extra;
pub mod generation;
pub mod jail;
pub mod recipe;
pub mod shell;
pub mod store;
mod support;
pub mod user;

#[cfg(not(test))]
pub const UID: Option<u32> = Some(0);
#[cfg(test)]
pub const UID: Option<u32> = None;

#[cfg(not(test))]
pub const GID: Option<u32> = Some(0);
#[cfg(test)]
pub const GID: Option<u32> = None;

pub mod cache {
    pub use cached_path::{Cache, CacheBuilder, Options};
}

pub mod version {
    pub use semver::{Version, VersionReq};
}

pub mod url {
    pub use url::Url;
}

pub mod config {
    use std::{
        error::Error,
        fs,
        os::unix,
        path::{Path, PathBuf},
    };

    use serde::{Deserialize, Serialize};
    use url::Url;

    use crate::{GID, UID};

    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
    pub struct Config {
        path: PathBuf,
        caches: Vec<Url>,
    }

    impl Config {
        pub fn init<P: AsRef<Path>>(path: P, caches: Vec<Url>) -> Result<Self, Box<dyn Error>> {
            let config = Self {
                caches,
                path: path.as_ref().to_owned(),
            };
            let bytes = toml::to_vec(&config)?;
            fs::write(&config.path, bytes)?;
            unix::fs::chown(&config.path, UID, GID)?;

            Ok(config)
        }

        pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
            let bytes = fs::read(path)?;
            let config = toml::from_slice(&bytes)?;
            Ok(config)
        }

        pub fn add_cache(&mut self, cache: Url) {
            self.caches.push(cache);
        }

        pub fn remove_cache(&mut self, index: usize) -> Url {
            self.caches.remove(index)
        }

        pub fn flush(&self) -> Result<(), Box<dyn Error>> {
            let bytes = toml::to_vec(&self)?;
            fs::remove_file(&self.path)?;
            fs::write(&self.path, bytes)?;
            unix::fs::chown(&self.path, UID, GID)?;
            Ok(())
        }

        pub fn caches(&self) -> &Vec<Url> {
            &self.caches
        }

        pub fn to_caches(self) -> Vec<Url> {
            self.caches
        }
    }
}
