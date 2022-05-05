use semver::{Comparator, Op, Version, VersionReq};

/// Filesystem manipulation operations.
pub mod fs;
/// Hashing operations.
pub mod hash;
/// Memory operations.
pub mod mem;
/// Path manipulation.
pub mod path;
/// [serde] persistation.
pub mod persist;
/// Styling.
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

/// Returns the exact [VersionReq] of a [Version].
///
/// # Example
///
/// ```
/// use semver::{Version, VersionReq};
/// use hua_core::extra;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let version = Version::new(0, 1, 1);
/// let req = extra::exact_version_req(version);
///
/// assert_eq!(req, VersionReq::parse("=0.1.1")?);
/// # Ok(())
/// # }
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

/// Unicode string slice operations.
pub mod str {

    /// Parses a [str] of hexadecimal numbers into a [Vec<u8>]
    ///
    /// # Example
    ///
    /// ```
    /// use hua_core::extra::str;
    /// # fn main() {
    /// let hex_asm = "a2b0";
    /// let vec = str::parse_hex(hex_asm);
    ///
    /// assert_eq!(vec, vec![162, 176]);
    /// # }
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
