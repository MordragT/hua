use core::slice::SlicePattern;
use std::{
    fs::{self, File, OpenOptions},
    io::BufWriter,
    path::{Path, PathBuf},
};

use reqwest::{blocking::Client, Url};
use rustbreak::PathDatabase;
use temp_dir::TempDir;

use crate::{error::Result, extra::Remote, persist::Pot, store::Packages, Error, Package, Store};

/// Filename of the cache database
pub const SOURCES_DB: &str = "sources.db";

/// A cache client to retrieve remote packages.
#[derive(Debug)]
pub struct CacheClient {
    client: Client,
    sources: PathDatabase<Vec<String>, Pot>,
    path: PathBuf,
}

impl CacheClient {
    /// Create a cache client at the specifed path.
    pub fn create_at_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let client = Client::new();
        let sources = PathDatabase::create_at_path(path.join(SOURCES_DB), Vec::new())?;

        Ok(Self {
            client,
            sources,
            path,
        })
    }

    /// Opens the cache client at the specified path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let client = Client::new();
        let sources = PathDatabase::load_from_path(path.join(SOURCES_DB))?;

        Ok(Self {
            client,
            sources,
            path,
        })
    }

    /// Create a cache client a the specified path with the given sources.
    pub fn from_sources<P: AsRef<Path>>(path: P, sources: &[String]) -> Result<Self> {
        let path = path.as_ref().to_owned();
        let client = Client::new();
        let sources = PathDatabase::create_at_path(path.join(SOURCES_DB), sources.to_owned())?;

        Ok(Self {
            client,
            sources,
            path,
        })
    }

    // TODO get and search packages must also get its dependencies

    /// Downloads the remote package and stores it inside the Store.
    pub fn download(&self, package: Remote<Package>, store: &mut Store) -> Result<u64> {
        // let path = store.create_package_path(&package.data, hash)
        // let url = package.source.join(
        //     package
        //         .data
        //         .path
        //         .as_os_str()
        //         .to_str()
        //         .ok_or(Error::OsStringConversion)?,
        // )?;

        // let mut response = self.client.get(url).send()?;

        // let temp_dir = TempDir::new()?;
        // let path = temp_dir.child("package");
        // fs::create_dir(&path)?;
        // let mut writer = BufWriter::new(File::open(&path)?);
        // response.copy_to(&mut writer)?;

        // let mut package = package.data;
        // package.path = path;

        todo!()
    }

    /// Gets the first matching package from the different sources.
    pub fn get(&self, hash: &u64) -> Result<Option<Remote<Package>>> {
        // TODO use async for concurrent download of metadata

        // self.read_sources(|vec| -> Result<Option<Remote<Package>>> {
        //     for source in vec {
        //         let mut packages_db = source.to_owned();
        //         packages_db.push_str("packages.db");

        //         let response = self.client.get(packages_db).send()?;
        //         let bytes = response.bytes()?;
        //         let mut packages: Packages = pot::from_slice(bytes.as_slice())?;
        //         if let Some(package) = packages.remove(hash) {
        //             let url = Url::parse(&source)?;
        //             return Ok(Some(Remote::new(package, url)));
        //         }
        //     }

        //     Ok(None)
        // })?
        todo!()
    }

    /// Searches for all matching packages from the different sources.
    pub fn search(&self, name: &str) -> Result<Vec<Remote<Package>>> {
        // TODO use async for concurrent download of metadata

        self.read_sources(|vec| -> Result<Vec<Remote<Package>>> {
            let dbs = vec
                .iter()
                .map(|source| {
                    let mut packages_db = source.to_owned();
                    packages_db.push_str("packages.db");

                    let response = self.client.get(packages_db).send()?;
                    let bytes = response.bytes()?;
                    let packages: Packages = pot::from_slice(bytes.as_slice())?;
                    Ok(Remote::new(packages, Url::parse(&source)?))
                })
                .collect::<Result<Vec<Remote<Packages>>>>()?;

            let packages = dbs
                .into_iter()
                .filter_map(|remote| {
                    if let Some(package) = remote.data.search(name) {
                        Some(Remote::new(package.clone(), remote.source))
                    } else {
                        None
                    }
                })
                .collect::<Vec<Remote<Package>>>();

            Ok(packages)
        })?
    }

    /// Returns the path of the cache client.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Dispatches a task over the clients sources.
    pub fn read_sources<T, R>(&self, task: T) -> Result<R>
    where
        T: FnOnce(&Vec<String>) -> R,
    {
        let res = self.sources.read(|vec| task(vec))?;
        Ok(res)
    }

    /// Inserts a source.
    pub fn insert_source(&mut self, source: &str) -> Result<()> {
        self.sources.write(|vec| vec.push(source.to_owned()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use temp_dir::TempDir;

    use super::{CacheClient, SOURCES_DB};

    #[test]
    fn cache_client_create_at_path() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("cache-client");

        let _cache_client = CacheClient::create_at_path(&path).unwrap();

        let db_file = path.join(SOURCES_DB);

        assert!(db_file.exists());
        assert!(db_file.is_file());
    }

    #[test]
    fn cache_client_open_ok() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("cache-client");

        let _cache_client = CacheClient::create_at_path(&path).unwrap();
        let _cache_client = CacheClient::open(&path).unwrap();
    }

    #[test]
    fn cache_client_open_err() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.child("cache-client");

        let res = CacheClient::open(&path);

        assert!(res.is_err());
    }
}
