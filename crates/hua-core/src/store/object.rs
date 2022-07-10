use super::ObjectId;
use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};
use url::Url;

/// A binary object.
/// For example:
/// - ELF executable
/// - txt file
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blob {
    /// The relative path of the [Blob].
    pub path: RelativePathBuf,
}

impl Blob {
    /// Creates a new [Blob].
    pub fn new(path: RelativePathBuf) -> Self {
        Self { path }
    }

    /// Converts a [Blob] to a [PathBuf].
    pub fn to_path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        self.path.to_path(base)
    }
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blob: {}", &self.path)
    }
}

/// A directory which can contain other [Object].
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tree {
    /// The relative path of the [Tree].
    pub path: RelativePathBuf,
    /// The children inside the `path` of the tree.
    pub children: Vec<ObjectId>,
}

impl Tree {
    /// Creates a new [Tree].
    pub fn new(path: RelativePathBuf, children: Vec<ObjectId>) -> Self {
        Self { path, children }
    }

    /// Converts a [Tree] to a [PathBuf].
    pub fn to_path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        self.path.to_path(base)
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tree: {}\n{:#?}", &self.path, &self.children)
    }
}

/// A link to another [Object].
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Link {
    /// The relative path of the [Link].
    pub link: RelativePathBuf,
    /// The source [Object] of the [Link].
    pub source: ObjectId,
}

impl Link {
    /// Creates a new [Link].
    pub fn new(link: RelativePathBuf, source: ObjectId) -> Self {
        Self { link, source }
    }

    /// Converts a [Link] to a [PathBuf].
    pub fn to_path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        self.link.to_path(base)
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Link: {} <- {}", &self.link, &self.source)
    }
}

/// The kind of an [Object].
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq)]
pub enum ObjectKind {
    Tree,
    Blob,
    Link,
}

/// A Object inside a [Package](super::Package).
/// Every file, folder or link is a [Object].
#[allow(missing_docs)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Object {
    Tree(Tree),
    Link(Link),
    Blob(Blob),
}

impl Object {
    /// The [ObjectKind] of an [Object].
    pub fn kind(&self) -> ObjectKind {
        match &self {
            Self::Tree(_) => ObjectKind::Tree,
            Self::Blob(_) => ObjectKind::Blob,
            Self::Link(_) => ObjectKind::Link,
        }
    }

    /// Replaces the [path](RelativePathBuf) of an [Object].
    pub fn replace_path(&mut self, path: RelativePathBuf) -> RelativePathBuf {
        match self {
            Self::Link(_) => todo!(),
            Self::Tree(tree) => std::mem::replace(&mut tree.path, path),
            Self::Blob(blob) => std::mem::replace(&mut blob.path, path),
        }
    }

    /// Converts an [Object] to a [PathBuf].
    pub fn to_path<P: AsRef<Path>>(&self, base: P) -> PathBuf {
        match &self {
            Self::Tree(tree) => tree.to_path(base),
            Self::Blob(blob) => blob.to_path(base),
            Self::Link(link) => link.to_path(base),
        }
    }

    /// Converts an [Object] to a [Url].
    pub fn to_url(&self, url: &Url) -> Url {
        match &self {
            Self::Tree(tree) => url.join(tree.path.as_str()).unwrap(),
            Self::Blob(blob) => url.join(blob.path.as_str()).unwrap(),
            Self::Link(link) => url.join(link.link.as_str()).unwrap(),
        }
    }

    /// Borrows a [Object] as [Tree] if it is one.
    pub fn as_tree(&self) -> Option<&Tree> {
        match &self {
            Self::Tree(tree) => Some(tree),
            _ => None,
        }
    }

    /// Checks if a [Object] is a [Tree].
    pub fn is_tree(&self) -> bool {
        self.kind() == ObjectKind::Tree
    }

    /// Converts a [Object] into a [Tree] if it is one.
    pub fn into_tree(self) -> Option<Tree> {
        match self {
            Self::Tree(tree) => Some(tree),
            _ => None,
        }
    }

    /// Borrows a [Object] as [Blob] if it is one.
    pub fn as_blob(&self) -> Option<&Blob> {
        match &self {
            Self::Blob(blob) => Some(blob),
            _ => None,
        }
    }

    /// Checks if a [Object] is a [Blob].
    pub fn is_blob(&self) -> bool {
        self.kind() == ObjectKind::Blob
    }

    /// Converts a [Object] into a [Blob] if it is one.
    pub fn into_blob(self) -> Option<Blob> {
        match self {
            Self::Blob(blob) => Some(blob),
            _ => None,
        }
    }

    /// Borrows a [Object] as [Link] if it is one.
    pub fn as_link(&self) -> Option<&Link> {
        match &self {
            Self::Link(link) => Some(link),
            _ => None,
        }
    }

    /// Checks if a [Object] is a [Link].
    pub fn is_link(&self) -> bool {
        self.kind() == ObjectKind::Link
    }

    /// Converts a [Object] into a [Link] if it is one.
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

/// Contains all [objects](Object) inside a [Store](super::Store).
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Objects {
    nodes: HashMap<ObjectId, Object>,
}

impl Objects {
    /// Creates a new [Objects] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if the [Object] is present with the [ObjectId].
    pub fn contains(&self, id: &ObjectId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Gets the reference to an [Object] with a [ObjectId].
    pub fn get(&self, id: &ObjectId) -> Option<&Object> {
        self.nodes.get(id)
    }

    /// Gets the mutable reference to an [Object] with a [ObjectId].
    pub fn get_mut(&mut self, id: &ObjectId) -> Option<&mut Object> {
        self.nodes.get_mut(id)
    }

    /// Get the reference to an [Object] with a [ObjectId] unchecked.
    pub unsafe fn get_unchecked(&self, id: &ObjectId) -> &Object {
        self.get(id).unwrap_unchecked()
    }

    /// Get the mutable reference to an [Object] with a [ObjectId] unchecked.
    pub unsafe fn get_mut_unchecked(&mut self, id: &ObjectId) -> &mut Object {
        self.get_mut(id).unwrap_unchecked()
    }

    /// Get multiple references to an [Object] with an [Iterator] of [object ids](ObjectId).
    pub fn get_multiple<'a>(
        &self,
        ids: impl IntoIterator<Item = &'a ObjectId>,
    ) -> impl Iterator<Item = (&'a ObjectId, &Object)> {
        ids.into_iter().map(move |id| {
            let object = unsafe { self.get_unchecked(id) };
            (id, object)
        })
    }

    /// Insert an [Object] with its [key](ObjectId).
    pub fn insert(&mut self, object_id: ObjectId, object: Object) -> Option<Object> {
        self.nodes.insert(object_id, object)
    }

    /// Remove a [ObjectId] returning the corresponding [Object].
    pub fn remove(&mut self, id: &ObjectId) -> Option<Object> {
        self.nodes.remove(id)
    }

    /// Remove multiple [objects](Object) defined by an [Iterator] of [object ids](ObjectId).
    pub fn remove_multiple<'a>(
        &'a mut self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = Option<(ObjectId, Object)>> + 'a {
        ids.into_iter()
            .map(|id| self.nodes.remove(id).map(|object| (*id, object)))
    }

    /// Remove multiple [objects](Object) defined by an [Iterator] of [object ids](ObjectId) unchecked.
    pub unsafe fn remove_multiple_unchecked<'a>(
        &'a mut self,
        ids: impl IntoIterator<Item = &'a ObjectId> + 'a,
    ) -> impl Iterator<Item = (ObjectId, Object)> + 'a {
        ids.into_iter().map(|id| {
            let object = self.nodes.remove(id);
            (*id, object.unwrap_unchecked())
        })
    }

    /// Get multiple references to [blobs](Blob) owned by multiple [objects](Object).
    pub fn get_multiple_blobs<'a>(
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
    /// Get multiple cloned [blobs](Blob) owned by multiple [objects](Object).
    pub fn get_multiple_blobs_cloned<'a>(
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

    #[deprecated]
    #[allow(missing_docs)]
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

    #[deprecated]
    #[allow(missing_docs)]
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
}
