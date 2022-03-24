use semver::{Version, VersionReq};

use crate::{components::OptionalComponentPaths, error::*};
use crate::{Component, Requirement, Store};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::path::{Path, PathBuf};

// pub trait PackageMetadata {
//     fn name(&self) -> &String;
//     fn version(&self) -> &String;
//     fn hash(&self) -> &u64;
//     fn source(&self) -> &Source;
// }

// pub struct RemotePackage {
//     pub name: String,
//     pub version: String,
//     pub hash: u64,
//     pub url: Url,
// }

// impl PackageMetadata for RemotePackage {
//     fn name(&self) -> &String {
//         &self.name
//     }

//     fn version(&self) -> &String {
//         &self.version
//     }

//     fn hash(&self) -> &u64 {
//         todo!()
//     }

//     fn source(&self) -> &Source {
//         &Source::Remote(self.url)
//     }
// }

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub provides: HashSet<Component>,
}

impl Hash for Package {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.version.hash(state);
        for component in &self.provides {
            component.hash(state);
        }
    }
}

// impl PackageMetadata for Package {
//     fn name(&self) -> &String {
//         &self.name
//     }

//     fn version(&self) -> &String {
//         &self.version
//     }

//     fn hash(&self) -> &u64 {
//         todo!()
//     }

//     fn source(&self) -> &Source {
//         &Source::Local(self.path)
//     }
// }

impl Package {
    /// Creates a new package
    pub fn new(name: &str, version: Version, provides: HashSet<Component>) -> Self {
        Self {
            name: name.to_owned(),
            version,
            provides,
        }
    }

    /// Creates a dependency from a package
    pub fn into_requirement(
        self,
        version_req: VersionReq,
        requires: HashSet<Requirement>,
    ) -> Requirement {
        Requirement::new(self.name, version_req, self.provides)
    }

    // pub fn name_version_hash(&self) -> Result<String> {
    //     let path_str = self.path.to_str().ok_or(Error::OsStringConversion)?;
    //     Ok(format!("{}-{}-{}", self.name, self.version, path_str))
    // }

    // TODO use HashSet<Component> instead and link them

    // pub fn optional_component_paths(&self) -> OptionalComponentPaths {
    //     OptionalComponentPaths::new(self.binary(), self.config(), self.library(), self.share())
    // }

    // pub fn binary(&self) -> Option<PathBuf> {
    //     let binary = self.path.join("bin");
    //     if binary.exists() {
    //         Some(binary)
    //     } else {
    //         None
    //     }
    // }

    // pub fn config(&self) -> Option<PathBuf> {
    //     let config = self.path.join("cfg");
    //     if config.exists() {
    //         Some(config)
    //     } else {
    //         None
    //     }
    // }

    // pub fn library(&self) -> Option<PathBuf> {
    //     let library = self.path.join("lib");
    //     if library.exists() {
    //         Some(library)
    //     } else {
    //         None
    //     }
    // }

    // pub fn share(&self) -> Option<PathBuf> {
    //     let share = self.path.join("share");
    //     if share.exists() {
    //         Some(share)
    //     } else {
    //         None
    //     }
    // }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}-{}", self.name, self.version))
    }
}
