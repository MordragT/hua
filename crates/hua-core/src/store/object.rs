use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

use super::{ObjectId, PackageId};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blob {
    pub path: RelativePathBuf,
}

impl Blob {
    pub fn new(path: RelativePathBuf) -> Self {
        Self { path }
    }

    pub fn to_path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        self.path.to_path(base)
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.path)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tree {
    pub path: RelativePathBuf,
    pub children: Vec<ObjectId>,
}

impl Tree {
    pub fn new(path: RelativePathBuf, children: Vec<ObjectId>) -> Self {
        Self { path, children }
    }

    pub fn to_path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        self.path.to_path(base)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Link {
    pub link: RelativePathBuf,
    pub source: ObjectId,
}

impl Link {
    pub fn new(link: RelativePathBuf, source: ObjectId) -> Self {
        Self { link, source }
    }

    pub fn to_path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        self.link.to_path(base)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ObjectKind {
    Tree,
    Blob,
    Link,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Object {
    Tree(Tree),
    Link(Link),
    Blob(Blob),
}

impl Object {
    pub fn kind(&self) -> ObjectKind {
        match &self {
            Self::Tree(_) => ObjectKind::Tree,
            Self::Blob(_) => ObjectKind::Blob,
            Self::Link(_) => ObjectKind::Link,
        }
    }

    pub fn replace_path(&mut self, path: RelativePathBuf) -> RelativePathBuf {
        match self {
            Self::Link(_) => todo!(),
            Self::Tree(tree) => std::mem::replace(&mut tree.path, path),
            Self::Blob(blob) => std::mem::replace(&mut blob.path, path),
        }
    }

    pub fn to_path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        match &self {
            Self::Tree(tree) => tree.to_path(base),
            Self::Blob(blob) => blob.to_path(base),
            Self::Link(link) => link.to_path(base),
        }
    }

    pub fn as_tree(&self) -> Option<&Tree> {
        match &self {
            Self::Tree(tree) => Some(tree),
            _ => None,
        }
    }

    pub fn is_tree(&self) -> bool {
        self.kind() == ObjectKind::Tree
    }

    pub fn into_tree(self) -> Option<Tree> {
        match self {
            Self::Tree(tree) => Some(tree),
            _ => None,
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

    pub fn as_link(&self) -> Option<&Link> {
        match &self {
            Self::Link(link) => Some(link),
            _ => None,
        }
    }

    pub fn is_link(&self) -> bool {
        self.kind() == ObjectKind::Link
    }

    pub fn into_link(self) -> Option<Link> {
        match self {
            Self::Link(link) => Some(link),
            _ => None,
        }
    }
}

impl Ord for Object {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self, other) {
            (Object::Link(_), Object::Tree(_)) => Ordering::Greater,
            (Object::Link(_), Object::Blob(_)) => Ordering::Greater,
            (Object::Link(link), Object::Link(other)) => link.cmp(other),
            (Object::Tree(_), Object::Link(_)) => Ordering::Less,
            (Object::Tree(_), Object::Blob(_)) => Ordering::Greater,
            (Object::Tree(tree), Object::Tree(other)) => tree.cmp(other),
            (Object::Blob(_), Object::Link(_)) => Ordering::Less,
            (Object::Blob(_), Object::Tree(_)) => Ordering::Less,
            (Object::Blob(blob), Object::Blob(other)) => blob.cmp(other),
        }
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Tree> for Object {
    fn from(tree: Tree) -> Self {
        Self::Tree(tree)
    }
}

impl From<Blob> for Object {
    fn from(blob: Blob) -> Self {
        Self::Blob(blob)
    }
}

impl From<Link> for Object {
    fn from(link: Link) -> Self {
        Self::Link(link)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectExt {
    pub links: HashMap<PackageId, Link>,
    // pub package_id: PackageId,
}

impl ObjectExt {
    pub fn new() -> Self {
        Self {
            links: HashMap::new(),
            // package_id,
        }
    }
    pub fn with_links(links: HashMap<PackageId, Link>) -> Self {
        Self { links }
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

    pub unsafe fn get_ext_mut_unchecked(&mut self, id: &ObjectId) -> &mut ObjectExt {
        self.get_ext_mut(id).unwrap_unchecked()
    }

    pub fn get(&self, id: &ObjectId) -> Option<&Object> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: &ObjectId) -> Option<&mut Object> {
        self.nodes.get_mut(id)
    }

    pub unsafe fn get_unchecked(&self, id: &ObjectId) -> &Object {
        self.get(id).unwrap_unchecked()
    }

    pub fn get_full(&self, id: &ObjectId) -> Option<(&Object, &ObjectExt)> {
        if let Some(object) = self.get(id) {
            self.get_ext(id).map(|ext| (object, ext))
        } else {
            None
        }
    }

    pub fn get_full_mut(&mut self, id: &ObjectId) -> Option<(&mut Object, &mut ObjectExt)> {
        if let Some(object) = self.nodes.get_mut(id) {
            self.extensions.get_mut(id).map(|ext| (object, ext))
        } else {
            None
        }
    }

    pub unsafe fn get_full_unchecked(&self, id: &ObjectId) -> (&Object, &ObjectExt) {
        self.get_full(id).unwrap_unchecked()
    }

    pub unsafe fn get_full_mut_unchecked(
        &mut self,
        id: &ObjectId,
    ) -> (&mut Object, &mut ObjectExt) {
        self.get_full_mut(id).unwrap_unchecked()
    }

    pub fn remove_objects<'a>(
        &'a mut self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = Option<(ObjectId, Object, ObjectExt)>> + 'a {
        ids.into_iter().map(|id| {
            let object = self.nodes.remove(id);
            let ext = self.extensions.remove(id);

            if let Some(object) = object && let Some(ext) = ext {
                Some((*id, object, ext))
            } else {
                None
            }
        })
    }

    pub unsafe fn remove_objects_unchecked<'a>(
        &'a mut self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = (ObjectId, Object, ObjectExt)> + 'a {
        ids.into_iter().map(|id| {
            let object = self.nodes.remove(id);
            let ext = self.extensions.remove(id);

            (*id, object.unwrap_unchecked(), ext.unwrap_unchecked())
        })
    }
    pub fn read_objects<'a, P, R>(
        &'a self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
        mut predicate: P,
    ) -> impl Iterator<Item = R> + 'a
    where
        P: FnMut(&Object, &ObjectExt) -> R + 'a,
    {
        ids.into_iter().map(move |id| {
            let (object, ext) = unsafe { self.get_full_unchecked(id) };
            predicate(object, ext)
        })
    }

    pub fn read_objects_mut<'a, P, R>(
        &'a mut self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
        mut predicate: P,
    ) -> impl Iterator<Item = R> + 'a
    where
        P: FnMut(&mut Object, &mut ObjectExt) -> R + 'a,
    {
        ids.into_iter().map(move |id| {
            let (object, ext) = unsafe { self.get_full_mut_unchecked(&id) };
            predicate(object, ext)
        })
    }

    pub fn insert(&mut self, object_id: ObjectId, object: Object) -> Option<Object> {
        if let Some(old) = self.nodes.insert(object_id, object) {
            Some(old)
        } else {
            self.extensions.insert(object_id, ObjectExt::new());
            None
        }
    }

    pub fn insert_full(
        &mut self,
        object_id: ObjectId,
        object: Object,
        ext: ObjectExt,
    ) -> Option<Object> {
        if let Some(old) = self.nodes.insert(object_id, object) {
            Some(old)
        } else {
            self.extensions.insert(object_id, ext);
            None
        }
    }

    pub fn get_blobs<'a>(
        &'a self,
        ids: impl IntoIterator<Item = &'a ObjectId>,
    ) -> impl Iterator<Item = &'a Blob> {
        ids.into_iter().filter_map(|id| {
            if let Some(object) = self.nodes.get(id) {
                object.as_blob()
            } else {
                None
            }
        })
    }

    pub fn get_blobs_cloned<'a>(
        &'a self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = Blob> + 'a {
        ids.into_iter().filter_map(move |id| {
            if let Some(object) = self.nodes.get(id) {
                object.clone().into_blob()
            } else {
                None
            }
        })
    }

    // pub fn get_foreign_links<'a>(
    //     &'a self,
    //     package_id: &'a PackageId,
    // ) -> impl Iterator<Item = (PackageId, ObjectId, &'a Link)> + 'a {
    //     self.extensions
    //         .iter()
    //         .filter_map(|(id, ext)| {
    //             if ext.package_id == *package_id && ext.links.len() > 0 {
    //                 Some(
    //                     ext.links
    //                         .iter()
    //                         .map(|(package_id, link)| (*package_id, *id, link)),
    //                 )
    //             } else {
    //                 None
    //             }
    //         })
    //         .flatten()
    // }

    // pub fn insert_link(
    //     &mut self,
    //     src_object: &ObjectId,
    //     target_package: PackageId,
    //     link: PathBuf,
    // ) -> bool {
    //     if let Some(ext) = self.extensions.get_mut(src_object) {
    //         ext.links.insert(target_package, link);
    //         true
    //     } else {
    //         false
    //     }
    // }

    // pub fn remove_tree_children_links(
    //     &mut self,
    //     tree_id: &ObjectId,
    //     target_package: &PackageId,
    // ) -> Option<Vec<PathBuf>> {
    //     let children = match self.nodes.get(tree_id) {
    //         Some(Object::Tree { path: _, children }) => Some(children),
    //         _ => None,
    //     };

    //     let mut result = Vec::new();

    //     if let Some(children) = children {
    //         for child in children {
    //             let ext = unsafe { self.extensions.get_mut(&child).unwrap_unchecked() };
    //             if let Some(link) = ext.links.remove(target_package) {
    //                 result.push(link);
    //             }
    //         }
    //         Some(result)
    //     } else {
    //         None
    //     }
    // }
}
