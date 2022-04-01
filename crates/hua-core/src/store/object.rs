use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectKind {
    Tree,
    Blob,
    // Link,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blob {
    pub path: RelativePathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Object {
    Tree {
        path: RelativePathBuf,
        children: Vec<ObjectId>,
    },
    // Link {
    //     path: RelativePathBuf,
    //     to: ObjectId,
    // },
    Blob(Blob),
}

impl Object {
    pub fn kind(&self) -> ObjectKind {
        match &self {
            Self::Tree {
                path: _,
                children: _,
            } => ObjectKind::Tree,
            Self::Blob(Blob { path: _ }) => ObjectKind::Blob,
            // Self::Link { path: _, to: _ } => ObjectKind::Link,
        }
    }

    pub fn to_path<P: AsRef<Path>>(&self, parent: P) -> PathBuf {
        match &self {
            Self::Tree { path, children: _ }
            | Self::Blob(Blob { path })
            /*| Self::Link { path, to: _ }*/ => path.to_path(parent),
        }
    }

    pub fn as_blob(&self) -> Option<&Blob> {
        match &self {
            Self::Blob(blob) => Some(blob),
            _ => None,
        }
    }

    pub fn is_blob(&self) -> bool {
        self.kind() == ObjectKind::Blob
    }

    pub fn into_blob(self) -> Option<Blob> {
        match self {
            Self::Blob(blob) => Some(blob),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectExt {
    pub links: HashSet<PathBuf>,
    pub package_id: ObjectId,
}

impl ObjectExt {
    pub fn new(package_id: ObjectId) -> Self {
        Self {
            links: HashSet::new(),
            package_id,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ObjectId([u8; 32]);

impl ObjectId {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn truncate(&self) -> u64 {
        let mut res: [u8; 8] = Default::default();
        res.copy_from_slice(&self.0[0..8]);
        u64::from_be_bytes(res)
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl From<[u8; 32]> for ObjectId {
    fn from(hash: [u8; 32]) -> Self {
        ObjectId(hash)
    }
}

impl From<ObjectId> for Vec<u8> {
    fn from(id: ObjectId) -> Self {
        id.0.to_vec()
    }
}

impl TryFrom<Vec<u8>> for ObjectId {
    type Error = Vec<u8>;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        let hash = <[u8; 32]>::try_from(vec)?;
        Ok(ObjectId(hash))
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Objects {
    nodes: HashMap<ObjectId, Object>,
    extensions: HashMap<ObjectId, ObjectExt>,
}

impl Objects {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, id: &ObjectId) -> bool {
        self.nodes.contains_key(id)
    }

    pub fn get_ext(&self, id: &ObjectId) -> Option<&ObjectExt> {
        self.extensions.get(id)
    }

    pub fn get_ext_mut(&mut self, id: &ObjectId) -> Option<&mut ObjectExt> {
        self.extensions.get_mut(id)
    }

    pub fn get(&self, id: &ObjectId) -> Option<&Object> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: &ObjectId) -> Option<&mut Object> {
        self.nodes.get_mut(id)
    }

    pub fn get_full(&self, id: &ObjectId) -> Option<(&Object, &ObjectExt)> {
        if let Some(object) = self.get(id) {
            self.get_ext(id).map(|ext| (object, ext))
        } else {
            None
        }
    }

    pub fn insert_link(&mut self, id: &ObjectId, link: PathBuf) -> bool {
        if let Some(ext) = self.extensions.get_mut(id) {
            ext.links.insert(link);
            true
        } else {
            false
        }
    }

    pub fn insert(
        &mut self,
        package_id: ObjectId,
        object_id: ObjectId,
        object: Object,
    ) -> Option<Object> {
        if let Some(old) = self.nodes.insert(object_id, object) {
            Some(old)
        } else {
            self.extensions
                .insert(object_id, ObjectExt::new(package_id));
            None
        }
    }
}
