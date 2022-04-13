use semver::{Comparator, Op, Version, VersionReq};

pub mod collections;
pub mod fs;
pub mod hash;
pub mod mem;
pub mod path;
pub mod persist;

// pub struct Remote<T> {
//     pub data: T,
//     pub source: Url,
// }

// impl<T> Remote<T> {
//     pub fn new(data: T, source: Url) -> Self {
//         Self { data, source }
//     }
// }

// #[derive(Debug)]
// pub enum Source {
//     Local { path: PathBuf, checksum: u64 },
//     Http { url: Url, checksum: u64 },
// }

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
