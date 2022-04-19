use semver::{Comparator, Op, Version, VersionReq};

pub mod fs;
pub mod hash;
pub mod mem;
pub mod path;
pub mod persist;
pub mod style;

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

pub mod str {
    pub fn parse_hex(hex_asm: &str) -> Vec<u8> {
        let mut hex_bytes = hex_asm
            .as_bytes()
            .iter()
            .filter_map(|b| match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                b'A'..=b'F' => Some(b - b'A' + 10),
                _ => None,
            })
            .fuse();

        let mut bytes = Vec::new();
        while let (Some(h), Some(l)) = (hex_bytes.next(), hex_bytes.next()) {
            bytes.push(h << 4 | l)
        }
        bytes
    }
}
