use reqwest::Url;
use semver::{Comparator, Op, Version, VersionReq};
use std::{
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
};

pub mod fs;
pub mod mem;

pub struct Remote<T> {
    pub data: T,
    pub source: Url,
}

impl<T> Remote<T> {
    pub fn new(data: T, source: Url) -> Self {
        Self { data, source }
    }
}

#[derive(Debug)]
pub enum Source {
    Local { path: PathBuf, checksum: u64 },
    Http { url: Url, checksum: u64 },
}

pub fn exact_version_req(v: Version) -> VersionReq {
    VersionReq {
        comparators: vec![Comparator {
            op: Op::Exact,
            major: v.major,
            minor: Some(v.minor),
            patch: Some(v.patch),
            pre: v.pre,
        }],
    }
}

// TODO use File and Reader to hash contents not read_dir

/// Calculates a hash with all the files under the given path
pub fn hash_path<P: AsRef<Path>, H: Hasher>(path: P, state: &mut H) -> io::Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        path.read_dir()?
            .map(|res| match res {
                Ok(entry) => hash_path(entry.path(), state),
                Err(e) => Err(e.into()),
            })
            .collect::<io::Result<()>>()
    } else {
        std::fs::read(path)?.hash(state);
        Ok(())
    }
}
