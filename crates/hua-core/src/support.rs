use crate::{
    dependency::Requirement,
    recipe::Derivation,
    store::{object::Blob, package::LocalPackageSource},
};
use relative_path::RelativePathBuf;
use semver::{Version, VersionReq};
use std::{
    collections::{BTreeSet, HashSet},
    fs::{self, File},
    path::Path,
};

#[allow(dead_code)]
pub fn pkg_req_ver_prov<P: AsRef<Path>>(
    name: &str,
    path: P,
    requires: impl IntoIterator<Item = Requirement>,
    version: &str,
    provides: &str,
) -> LocalPackageSource {
    let path = path.as_ref();
    let package_lib_path = path.join("lib");
    fs::create_dir_all(&package_lib_path).unwrap();

    let lib_name = format!("{provides}.so");
    let lib_path = package_lib_path.join(&lib_name);

    let _lib = File::create(&lib_path).unwrap();

    let drv = Derivation::new(
        name.to_owned(),
        Version::parse(version).unwrap(),
        "Some package".to_owned(),
        1,
        1,
        String::new(),
        vec!["MIT".to_owned()],
        requires.into_iter().collect(),
        HashSet::new(),
        Vec::new(),
        String::new(),
        RelativePathBuf::new(),
    );

    LocalPackageSource::new(drv, path.to_owned())
}

#[allow(dead_code)]
pub fn pkg_prov<P: AsRef<Path>>(name: &str, path: P, provides: &str) -> LocalPackageSource {
    pkg_req_ver_prov(name, path, [], "1.0.0", provides)
}

#[allow(dead_code)]
pub fn pkg_req<P: AsRef<Path>>(
    name: &str,
    path: P,
    requires: impl IntoIterator<Item = Requirement>,
) -> LocalPackageSource {
    pkg_req_ver_prov(name, path, requires, "1.0.0", name)
}

#[allow(dead_code)]
pub fn pkg<P: AsRef<Path>>(name: &str, path: P) -> LocalPackageSource {
    pkg_req(name, path, BTreeSet::new())
}

#[allow(dead_code)]
pub fn pkg_ver<P: AsRef<Path>>(name: &str, path: P, version: &str) -> LocalPackageSource {
    pkg_req_ver_prov(name, path, [], version, name)
}

#[allow(dead_code)]
pub fn req(name: &str, version_req: &str) -> Requirement {
    req_comp(
        name,
        version_req,
        [Blob {
            path: RelativePathBuf::from(&format!("lib/{name}.so")),
        }],
    )
}

#[allow(dead_code)]
pub fn req_comp(
    name: &str,
    version_req: &str,
    objects: impl IntoIterator<Item = Blob>,
) -> Requirement {
    Requirement::new(
        name.to_owned(),
        VersionReq::parse(version_req).unwrap(),
        objects.into_iter().collect(),
    )
}
