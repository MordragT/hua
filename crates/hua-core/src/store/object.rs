use super::ObjectId;
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

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
        write!(f, "Blob: {}", &self.path)
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

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tree: {}\n{:#?}", &self.path, &self.children)
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

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Link: {} <- {}", &self.link, &self.source)
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

    pub fn relative_path(&self) -> &RelativePath {
        match &self {
            Self::Tree(tree) => &tree.path,
            Self::Blob(blob) => &blob.path,
            Self::Link(link) => &link.link,
        }
    }

    // pub fn to_url(&self, url: &Url) -> Url {
    //     match &self {
    //         Self::Tree(tree) => url.join(tree.path.as_str()).unwrap(),
    //         Self::Blob(blob) => url.join(blob.path.as_str()).unwrap(),
    //         Self::Link(link) => url.join(link.link.as_str()).unwrap(),
    //     }
    // }

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

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Link(link) => write!(f, "Object {link}"),
            Self::Tree(tree) => write!(f, "Object {tree}"),
            Self::Blob(blob) => write!(f, "Object {blob}"),
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Objects {
    nodes: HashMap<ObjectId, Object>,
}

impl Objects {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn contains(&self, id: &ObjectId) -> bool {
        self.nodes.contains_key(id)
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

    pub unsafe fn get_mut_unchecked(&mut self, id: &ObjectId) -> &mut Object {
        self.get_mut(id).unwrap_unchecked()
    }

    pub fn get_multiple<'a>(
        &self,
        ids: impl IntoIterator<Item = &'a ObjectId>,
    ) -> impl Iterator<Item = (&'a ObjectId, &Object)> {
        ids.into_iter().map(move |id| {
            let object = unsafe { self.get_unchecked(id) };
            (id, object)
        })
    }

    pub fn remove_objects<'a>(
        &'a mut self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = Option<(ObjectId, Object)>> + 'a {
        ids.into_iter()
            .map(|id| self.nodes.remove(id).map(|object| (*id, object)))
    }

    pub unsafe fn remove_objects_unchecked<'a>(
        &'a mut self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = (ObjectId, Object)> + 'a {
        ids.into_iter().map(|id| {
            let object = self.nodes.remove(id);
            (*id, object.unwrap_unchecked())
        })
    }
    pub fn read_objects<'a, P, R>(
        &'a self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
        mut predicate: P,
    ) -> impl Iterator<Item = R> + 'a
    where
        P: FnMut(&ObjectId, &Object) -> R + 'a,
    {
        ids.into_iter().map(move |id| {
            let object = unsafe { self.get_unchecked(id) };
            predicate(id, object)
        })
    }

    pub fn read_objects_mut<'a, P, R>(
        &'a mut self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
        mut predicate: P,
    ) -> impl Iterator<Item = R> + 'a
    where
        P: FnMut(&ObjectId, &mut Object) -> R + 'a,
    {
        ids.into_iter().map(move |id| {
            let object = unsafe { self.get_mut_unchecked(&id) };
            predicate(id, object)
        })
    }

    pub fn insert(&mut self, object_id: ObjectId, object: Object) -> Option<Object> {
        self.nodes.insert(object_id, object)
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

    pub fn get_trees_cloned<'a>(
        &'a self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = Tree> + 'a {
        ids.into_iter().filter_map(move |id| {
            if let Some(object) = self.nodes.get(id) {
                object.clone().into_tree()
            } else {
                None
            }
        })
    }

    pub fn get_blobs_ids_cloned<'a>(
        &'a self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = (Blob, ObjectId)> + 'a {
        ids.into_iter().filter_map(move |id| {
            if let Some(object) = self.nodes.get(id) {
                object.clone().into_blob().map(|blob| (blob, *id))
            } else {
                None
            }
        })
    }

    pub fn get_trees_ids_cloned<'a>(
        &'a self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = (Tree, ObjectId)> + 'a {
        ids.into_iter().filter_map(move |id| {
            if let Some(object) = self.nodes.get(id) {
                object.clone().into_tree().map(|tree| (tree, *id))
            } else {
                None
            }
        })
    }
}
