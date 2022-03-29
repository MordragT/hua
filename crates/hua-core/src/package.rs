use semver::{Version, VersionReq};

use crate::dependency::{Conflict, Conflicts};
use crate::{components::OptionalComponentPaths, error::*};
use crate::{Component, Requirement, Store};
use std::collections::{BTreeSet, HashSet};
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

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, Hash)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub provides: BTreeSet<Component>,
}

// impl Hash for Package {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.name.hash(state);
//         self.version.hash(state);
//         for component in &self.provides {
//             component.hash(state);
//         }
//     }
// }

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
    pub fn new(name: &str, version: Version, provides: BTreeSet<Component>) -> Self {
        Self {
            name: name.to_owned(),
            version,
            provides,
        }
    }

    /// Creates a dependency from a package
    pub fn into_requirement(self, version_req: VersionReq) -> Requirement {
        Requirement::new(self.name, version_req, self.provides)
    }
}

impl Conflicts for Package {
    fn conflicts<'a>(&'a self) -> &dyn Iterator<Item = Conflict<'a>> {
        &self
            .provides
            .iter()
            .map(|c| Conflict::Component(c))
            .chain(std::iter::once(Conflict::Name(&self.name)))
    }
}

impl fmt::Display for Package {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{}-{}", self.name, self.version))
    }
}
