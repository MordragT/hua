use hua_core::{Blob, Requirement, VersionReq};
use roc_std::{ReferenceCount, RocList, RocStr};

#[repr(C)]
pub struct RocRequirement {
    pub name: RocStr,
    pub versionReq: RocStr,
    pub blobs: RocList<RocStr>,
}

unsafe impl ReferenceCount for RocRequirement {
    fn increment(&self) {
        ReferenceCount::increment(&self.name);
        ReferenceCount::increment(&self.versionReq);
        ReferenceCount::increment(&self.blobs);
    }

    unsafe fn decrement(ptr: *const Self) {
        let req = &*ptr;

        ReferenceCount::decrement(&req.name as *const RocStr);
        ReferenceCount::decrement(&req.versionReq as *const RocStr);
        ReferenceCount::decrement(&req.blobs as *const RocList<RocStr>);
    }
}

impl Into<Requirement> for RocRequirement {
    fn into(self) -> Requirement {
        Requirement::new(
            self.name.to_string(),
            VersionReq::parse(&self.versionReq).unwrap(),
            self.blobs
                .into_iter()
                .map(|path_str| Blob::new(path_str.to_string().into()))
                .collect(),
        )
    }
}

#[repr(C)]
pub struct RocRecipe {
    pub name: RocStr,
    pub version: RocStr,
    pub desc: RocStr,
    pub archs: u8,
    pub platforms: u8,
    // TODO checksum
    pub source: RocStr,
    pub licenses: RocList<RocStr>,
    pub requires: RocList<RocRequirement>,
    pub requiresBuild: RocList<RocRequirement>,
    pub targetDir: RocStr,
    pub envs: RocList<Tuple<RocStr, RocStr>>,
    pub script: RocStr,
}

#[repr(C)]
pub struct Tuple<L, R> {
    pub left: L,
    pub right: R,
}

unsafe impl<L: ReferenceCount, R: ReferenceCount> ReferenceCount for Tuple<L, R> {
    fn increment(&self) {
        ReferenceCount::increment(&self.left);
        ReferenceCount::increment(&self.right);
    }

    unsafe fn decrement(ptr: *const Self) {
        let tuple = &*ptr;

        ReferenceCount::decrement(&tuple.left as *const L);
        ReferenceCount::decrement(&tuple.right as *const R);
    }
}
