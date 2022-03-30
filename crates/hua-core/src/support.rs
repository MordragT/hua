use crate::{Component, Package, Requirement};
use relative_path::RelativePathBuf;
use semver::{Version, VersionReq};
use std::{
    collections::BTreeSet,
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
) -> Package {
    let path = path.as_ref();
    let package_lib_path = path.join("lib");
    fs::create_dir_all(&package_lib_path).unwrap();

    let lib_name = format!("{provides}.so");
    let lib_path = package_lib_path.join(&lib_name);

    let _lib = File::create(&lib_path).unwrap();
    let mut provides = BTreeSet::new();
    provides.insert(Component::Library(RelativePathBuf::from(&format!(
        "lib/{lib_name}"
    ))));

    Package::new(
        name,
        Version::parse(version).unwrap(),
        provides.into_iter().collect(),
        requires.into_iter().collect(),
    )
}

#[allow(dead_code)]
pub fn pkg_prov<P: AsRef<Path>>(name: &str, path: P, provides: &str) -> Package {
    pkg_req_ver_prov(name, path, [], "1.0.0", provides)
}

#[allow(dead_code)]
pub fn pkg_req<P: AsRef<Path>>(
    name: &str,
    path: P,
    requires: impl IntoIterator<Item = Requirement>,
) -> Package {
    pkg_req_ver_prov(name, path, requires, "1.0.0", name)
}

#[allow(dead_code)]
pub fn pkg<P: AsRef<Path>>(name: &str, path: P) -> Package {
    pkg_req(name, path, BTreeSet::new())
}

#[allow(dead_code)]
pub fn pkg_ver<P: AsRef<Path>>(name: &str, path: P, version: &str) -> Package {
    pkg_req_ver_prov(name, path, [], version, name)
}

#[allow(dead_code)]
pub fn to_req(package: &Package) -> Requirement {
    package.clone().into_requirement()
}

#[allow(dead_code)]
pub fn req(name: &str, version_req: &str) -> Requirement {
    req_comp(
        name,
        version_req,
        [Component::Library(RelativePathBuf::from(&format!(
            "lib/{name}.so"
        )))],
    )
}

#[allow(dead_code)]
pub fn req_comp(
    name: &str,
    version_req: &str,
    components: impl IntoIterator<Item = Component>,
) -> Requirement {
    Requirement::new(
        name.to_owned(),
        VersionReq::parse(version_req).unwrap(),
        components.into_iter().collect(),
    )
}
