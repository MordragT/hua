use super::path;
use rs_merkle::{Hasher, MerkleTree};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read},
    path::Path,
};
use walkdir::WalkDir;

use crate::store::{
    id::{ObjectId, PackageId, RawId},
    object::{Blob, Tree},
    package::PackageSource,
};

/// Provides a Blake3 [Hasher] for the [MerkleTree].
#[derive(Clone)]
pub struct Blake3;

impl Hasher for Blake3 {
    type Hash = RawId;

    fn hash(data: &[u8]) -> Self::Hash {
        let mut hasher = blake3::Hasher::new();

        hasher.update(data);
        let hash = <[u8; 32]>::from(hasher.finalize());
        hash.into()
    }
}

pub fn root_hash(source: &PackageSource) -> io::Result<PackageId> {
    let pkg_hash = PackageHash::from_path(&source.path, source.name())?;
    Ok(pkg_hash.root)
}

// TODO: use PartialTrees and merge them when we get to upper directories

/// Calculates the hashes for the directories of an [crate::store::package::Package].
#[derive(Debug)]
pub struct PackageHash {
    /// The [PackageId] caclulated out of all [crate::store::Object] in the directory
    /// and the [crate::store::package::Package] name.
    pub root: PackageId,

    /// All [ObjectId] of all [Tree] in the directory.
    pub trees: BTreeMap<Tree, ObjectId>,

    /// All [ObjectId] of all [Blob] in the directory.
    pub blobs: BTreeMap<Blob, ObjectId>,
}

impl PackageHash {
    /// Calculates the hashes of an directory
    ///
    /// # Arguments
    ///
    /// - `path` - The path of the directory
    /// - `package_name` - The name of the package
    ///
    /// # Example
    ///
    /// ```no_run
    /// use hua_core::extra::hash::PackageHash;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let package_name = "make";
    /// let path = "./make-4.3";
    ///
    /// let hash = PackageHash::from_path(path, package_name)?;
    /// # Ok(())
    /// # }
    pub fn from_path<P: AsRef<Path>>(path: P, package_name: &str) -> io::Result<Self> {
        let mut trees = BTreeMap::new();
        let mut blobs = BTreeMap::new();

        let mut tree = MerkleTree::<Blake3>::new();
        let mut dir_children: Vec<RawId> = Vec::new();

        let root_path = path.as_ref().canonicalize()?;

        for entry in WalkDir::new(&root_path).contents_first(true) {
            let entry = entry?;
            let path = entry.path();

            if path == root_path {
                let hash = Blake3::hash(package_name.as_bytes());
                tree.insert(hash);
                tree.commit();

                return Ok(Self {
                    root: unsafe { tree.root().unwrap_unchecked() }.into(),
                    trees,
                    blobs,
                });
            } else if path.is_file() {
                let name = path.file_name().unwrap().to_str().unwrap();
                let mut bytes = <Vec<u8>>::from(name);

                let mut file = File::open(path)?;
                file.read(&mut bytes)?;

                let hash = Blake3::hash(&bytes);

                tree.insert(hash);
                dir_children.push(hash);

                let path = path::relative_path_between(&root_path, path)?;
                blobs.insert(Blob::new(path), hash.into());
            } else if path.is_dir() {
                let name = path.file_name().unwrap().to_str().unwrap();

                let hash = Blake3::hash(name.as_bytes());
                tree.insert(hash);
                tree.commit();

                let inner_children = std::mem::replace(&mut dir_children, Vec::new());
                let path = path::relative_path_between(&root_path, path)?;
                let root = {
                    // Calculate root from another tree so that subelemnts of parallel directories
                    // are not included
                    // Use partial trees
                    let mut tree = MerkleTree::<Blake3>::from_leaves(&inner_children);
                    tree.insert(hash);
                    tree.commit();
                    unsafe { tree.root().unwrap_unchecked() }
                };

                trees.insert(
                    Tree::new(
                        path,
                        inner_children
                            .into_iter()
                            .map(Into::into)
                            .collect::<Vec<ObjectId>>(),
                    ),
                    root.into(),
                );
            } else if path.is_symlink() {
                todo!()
            }
        }
        unreachable!()
    }
}

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

    // #[test]
    // fn package_hash_from_path_err() {
    //     let _err = PackageHash::from_path("..", "pkg").unwrap_err();
    // }
}
