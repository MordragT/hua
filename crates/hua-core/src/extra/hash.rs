use super::path;
use rs_merkle::{Hasher, MerkleTree};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read},
    path::Path,
};
use walkdir::WalkDir;

use crate::store::{Blob, Object, ObjectId};

#[derive(Clone)]
pub struct Blake3;

impl Hasher for Blake3 {
    type Hash = ObjectId;

    fn hash(data: &[u8]) -> Self::Hash {
        let mut hasher = blake3::Hasher::new();

        hasher.update(data);
        let hash = <[u8; 32]>::from(hasher.finalize());
        hash.into()
    }
}

// TODO use File and Reader to hash contents not read_dir

/// Calculates a hash with all the files under the given path
// pub fn hash_path<P: AsRef<Path>, H: Hasher>(path: P, state: &mut H) -> io::Result<()> {
//     let path = path.as_ref();
//     if path.is_dir() {
//         path.read_dir()?
//             .map(|res| match res {
//                 Ok(entry) => hash_path(entry.path(), state),
//                 Err(e) => Err(e.into()),
//             })
//             .collect::<io::Result<()>>()
//     } else {
//         std::fs::read(path)?.hash(state);
//         Ok(())
//     }
// }

// TODO: use PartialTrees and merge them when we get to upper directories

#[derive(Debug)]
pub struct PackageHash {
    pub root: ObjectId,
    pub children: HashMap<ObjectId, Object>,
}

impl PackageHash {
    pub fn from_path<P: AsRef<Path>>(path: P, package_name: &str) -> io::Result<Self> {
        let mut children = HashMap::new();
        let mut tree = MerkleTree::<Blake3>::new();
        let mut dir_children = Vec::new();

        let root_path = path.as_ref();

        for entry in WalkDir::new(root_path).contents_first(true) {
            let entry = entry?;
            let path = entry.path();

            if path == root_path {
                let hash = Blake3::hash(package_name.as_bytes());
                tree.insert(hash);
                tree.commit();

                return Ok(Self {
                    root: unsafe { tree.root().unwrap_unchecked() },
                    children,
                });
            } else if path.is_file() {
                let name = path.file_name().unwrap().to_str().unwrap();
                let mut bytes = <Vec<u8>>::from(name);

                let mut file = File::open(path)?;
                file.read(&mut bytes)?;

                let hash = Blake3::hash(&bytes);

                tree.insert(hash);
                dir_children.push(hash);

                let path = path::relative_path_between(root_path, path)?;
                children.insert(hash, Object::Blob(Blob { path }));
            } else if path.is_dir() {
                let name = path.file_name().unwrap().to_str().unwrap();

                let hash = Blake3::hash(name.as_bytes());
                tree.insert(hash);
                tree.commit();

                let inner_children = std::mem::replace(&mut dir_children, Vec::new());
                let path = path::relative_path_between(root_path, path)?;
                let root = {
                    // Calculate root from another tree so that subelemnts of parallel directories
                    // are not included
                    let mut tree = MerkleTree::<Blake3>::from_leaves(&inner_children);
                    tree.insert(hash);
                    tree.commit();
                    unsafe { tree.root().unwrap_unchecked() }
                };

                children.insert(
                    root,
                    Object::Tree {
                        path,
                        children: inner_children,
                    },
                );
            }
        }
        unreachable!()
    }
}

// pub fn hash_path_to_root_leaves<P: AsRef<Path>>(path: P) -> io::Result<(ObjectId, Vec<ObjectId>)> {
//     use io::{Error, ErrorKind};
//     let mut leaves = Vec::new();
//     let mut tree = MerkleTree::<Blake3>::new();

//     let root = path.as_ref();

//     if root.is_file() {
//         return Err(Error::new(ErrorKind::InvalidData, "Expected Directory"));
//     }

//     for entry in WalkDir::new(root).contents_first(true) {
//         let entry = entry?;
//         let path = entry.path();

//         if path == root {
//             break;
//         } else if path.is_file() {
//             let mut file = File::open(path)?;
//             let mut buf = Vec::new();
//             file.read(&mut buf)?;

//             let id = Blake3::hash(buf.as_slice());
//             tree.insert(id);
//             leaves.push(id);
//         } else if path.is_dir() {
//             let name = path
//                 .file_name()
//                 .ok_or(Error::new(ErrorKind::Other, "Terminating path"))?
//                 .to_str()
//                 .ok_or(Error::new(ErrorKind::InvalidData, "Invalid Utf-8"))?;

//             let id = Blake3::hash(name.as_bytes());

//             tree.insert(id);
//             tree.commit();
//             let root = unsafe { tree.root().unwrap_unchecked() };

//             leaves.push(root);
//         }
//     }

//     Ok(tree.proof(leaf_indices))
// }

#[cfg(test)]
mod tests {
    use super::PackageHash;
    use std::fs::{self, File};
    use temp_dir::TempDir;

    #[test]
    fn package_hash_from_path_ok() {
        let temp_dir = TempDir::new().unwrap();

        let pkg_dir = temp_dir.child("pkg-12314152352");
        let bin_dir = pkg_dir.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();

        let file_path = bin_dir.join("some_file");
        let _file = File::create(&file_path).unwrap();

        let _ok = PackageHash::from_path(pkg_dir, "pkg").unwrap();
    }

    #[test]
    fn package_hash_from_path_err() {
        let _err = PackageHash::from_path("..", "pkg").unwrap_err();
    }
}
