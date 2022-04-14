use crate::{
    dependency::Requirement,
    recipe::Recipe,
    store::{object::Blob, LocalStore, STORE_PATH},
};
use cached_path::CacheBuilder;
use relative_path::RelativePathBuf;
use semver::{Version, VersionReq};
use snafu::prelude::*;
use std::{
    ffi::{c_size_t, CStr},
    os::raw::c_char,
    str::Utf8Error,
};

#[derive(Debug, Snafu)]

pub enum CFFIError {
    #[snafu(display("Utf8Error: {source}"))]
    Utf8Error { source: Utf8Error },
    #[snafu(display("SemVerError: {source}"))]
    SemVerError { source: semver::Error },
}

type CFFIResult<T> = Result<T, CFFIError>;

#[derive(Debug)]
#[repr(C)]
pub struct CRecipe {
    pub name: *const c_char,
    pub version: *const c_char,
    pub desc: *const c_char,
    pub archs: u8,
    pub platforms: u8,
    pub source: *const c_char,
    pub licenses: *const *const c_char,
    pub licenses_len: c_size_t,
    pub requires: *const CRequirement,
    pub requires_len: c_size_t,
    pub requires_build: *const CRequirement,
    pub requires_build_len: c_size_t,
    pub target_dir: *const c_char,
    pub envs: *const CStringTuple,
    pub envs_len: c_size_t,
    pub script: *const c_char,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CRequirement {
    pub name: *const c_char,
    pub version_req: *const c_char,
    pub blobs: *const *const c_char,
    pub blobs_len: c_size_t,
}

impl TryInto<Requirement> for CRequirement {
    type Error = CFFIError;

    fn try_into(self) -> Result<Requirement, Self::Error> {
        let name = unsafe { CStr::from_ptr(self.name) }
            .to_str()
            .context(Utf8Snafu)?
            .to_owned();
        let version_req_str = unsafe { CStr::from_ptr(self.version_req) }
            .to_str()
            .context(Utf8Snafu)?;
        let version_req = VersionReq::parse(version_req_str).context(SemVerSnafu)?;
        let objects = unsafe { std::slice::from_raw_parts(self.blobs, self.blobs_len) }
            .into_iter()
            .map(|blob| {
                let blob_string = unsafe { CStr::from_ptr(*blob) }
                    .to_str()
                    .context(Utf8Snafu)?
                    .to_owned();
                Ok(Blob::new(RelativePathBuf::from(blob_string)))
            })
            .collect::<Result<_, CFFIError>>()?;

        Ok(Requirement::new(name, version_req, objects))
    }
}

impl TryInto<Requirement> for &CRequirement {
    type Error = CFFIError;

    fn try_into(self) -> Result<Requirement, Self::Error> {
        (*self).try_into()
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CStringTuple {
    pub left: *const c_char,
    pub right: *const c_char,
}

impl<'a> TryInto<(&'a str, &'a str)> for CStringTuple {
    type Error = CFFIError;

    fn try_into(self) -> Result<(&'a str, &'a str), Self::Error> {
        let left = unsafe { CStr::from_ptr(self.left) }
            .to_str()
            .context(Utf8Snafu)?;
        let right = unsafe { CStr::from_ptr(self.right) }
            .to_str()
            .context(Utf8Snafu)?;
        Ok((left, right))
    }
}

impl<'a> TryInto<(&'a str, &'a str)> for &CStringTuple {
    type Error = CFFIError;

    fn try_into(self) -> Result<(&'a str, &'a str), Self::Error> {
        (*self).try_into()
    }
}

impl TryInto<(String, String)> for CStringTuple {
    type Error = CFFIError;

    fn try_into(self) -> Result<(String, String), Self::Error> {
        let left = unsafe { CStr::from_ptr(self.left) }
            .to_str()
            .context(Utf8Snafu)?
            .to_owned();
        let right = unsafe { CStr::from_ptr(self.right) }
            .to_str()
            .context(Utf8Snafu)?
            .to_owned();
        Ok((left, right))
    }
}

impl TryInto<(String, String)> for &CStringTuple {
    type Error = CFFIError;

    fn try_into(self) -> Result<(String, String), Self::Error> {
        (*self).try_into()
    }
}

fn inner_build_recipe<'a>(
    name: &CStr,
    version: &CStr,
    desc: &CStr,
    archs: u8,
    platforms: u8,
    source: &CStr,
    licenses: impl Iterator<Item = &'a CStr>,
    requires: impl Iterator<Item = &'a CRequirement>,
    requires_build: impl Iterator<Item = &'a CRequirement>,
    target_dir: &CStr,
    envs: impl Iterator<Item = &'a CStringTuple>,
    script: &CStr,
) -> CFFIResult<()> {
    let name = name.to_str().context(Utf8Snafu)?.to_owned();
    let version = Version::parse(version.to_str().context(Utf8Snafu)?).context(SemVerSnafu)?;
    let desc = desc.to_str().context(Utf8Snafu)?.to_owned();
    let source = source.to_str().context(Utf8Snafu)?.to_owned();
    let licenses = licenses
        .map(|license| Ok(license.to_str().context(Utf8Snafu)?.to_owned()))
        .collect::<CFFIResult<_>>()?;
    let requires = requires
        .map(|req| req.try_into())
        .collect::<CFFIResult<_>>()?;
    let requires_build = requires_build
        .map(|req| req.try_into())
        .collect::<CFFIResult<_>>()?;
    let target_dir = target_dir.to_str().context(Utf8Snafu)?.to_owned().into();
    let envs = envs
        .map(|tuple| tuple.try_into())
        .collect::<CFFIResult<Vec<(&str, &str)>>>()?;
    let script = script.to_str().context(Utf8Snafu)?.to_owned();

    let recipe = Recipe::new(
        name,
        version,
        desc,
        archs,
        platforms,
        source,
        licenses,
        requires,
        requires_build,
        target_dir,
    );

    let store = LocalStore::open(STORE_PATH).unwrap();
    let cache = CacheBuilder::new().build().unwrap();

    let (_package, _path) = recipe
        .fetch(&cache)
        .unwrap()
        .prepare_requirements(&store, envs.into_iter())
        .unwrap()
        .build(script.to_string())
        .unwrap();

    println!("Package created successfully!");
    Ok(())
}

#[no_mangle]
pub unsafe extern "C" fn build_recipe(recipe: CRecipe) {
    let CRecipe {
        name,
        version,
        desc,
        archs,
        platforms,
        source,
        licenses,
        licenses_len,
        requires,
        requires_len,
        requires_build,
        requires_build_len,
        target_dir,
        envs,
        envs_len,
        script,
    } = recipe;

    let name = CStr::from_ptr(name);
    let version = CStr::from_ptr(version);
    let desc = CStr::from_ptr(desc);
    let source = CStr::from_ptr(source);
    let licenses = std::slice::from_raw_parts(licenses, licenses_len as usize)
        .into_iter()
        .map(|license| CStr::from_ptr(*license));
    let requires = std::slice::from_raw_parts(requires, requires_len as usize).into_iter();
    let requires_build =
        std::slice::from_raw_parts(requires_build, requires_build_len as usize).into_iter();
    let target_dir = CStr::from_ptr(target_dir);
    let envs = std::slice::from_raw_parts(envs, envs_len).into_iter();
    let script = CStr::from_ptr(script);

    inner_build_recipe(
        name,
        version,
        desc,
        archs,
        platforms,
        source,
        licenses,
        requires,
        requires_build,
        target_dir,
        envs,
        script,
    )
    .unwrap();
}
