use crate::{Component, ComponentPaths};
use std::{
    collections::HashSet,
    fs,
    hash::Hash,
    io::{Error, ErrorKind, Result},
    os::unix,
    path::{Path, PathBuf},
};

// TODO: better naming
// TODO: tests
// TODO: return IntoIterator
// TODO: own Error and Result types

pub fn io_task_into<P, Q, T, R, F, C>(from: P, into: Q, task: &T, fold: &F) -> Result<C>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    T: Fn(&Path, &Path) -> std::io::Result<R>,
    R: Eq + Hash,
    F: Fn(&mut C, R),
    C: Default,
{
    let mut collector = C::default();
    inner_io_task_into(from.as_ref(), into.as_ref(), task, fold, &mut collector)?;
    Ok(collector)
}

fn inner_io_task_into<R, T, F, C>(
    from: &Path,
    into: &Path,
    task: &T,
    fold: &F,
    collector: &mut C,
) -> Result<()>
where
    T: Fn(&Path, &Path) -> std::io::Result<R>,
    R: Eq + Hash,
    F: Fn(&mut C, R),
    C: Default,
{
    let from = from.canonicalize()?;
    let name = from
        .file_name()
        .ok_or(Error::new(
            ErrorKind::UnexpectedEof,
            "Unexpected terminating path",
        ))?
        .to_str()
        .ok_or(Error::new(ErrorKind::InvalidData, "Invalid utf-8"))?;
    let into = into.join(name);

    if from.is_dir() {
        from.read_dir()?
            .map(|res| match res {
                Ok(entry) => {
                    if !into.exists() {
                        fs::create_dir(&into)?;
                    }
                    inner_io_task_into(&entry.path(), &into, task, fold, collector)
                }
                Err(e) => Err(e.into()),
            })
            .collect::<Result<()>>()
    } else {
        fold(collector, task(&from, &into)?);
        Ok(())
    }
}

pub fn io_task_to<P, Q, T, R, F, C>(from: P, to: Q, task: &T, fold: &F) -> Result<C>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    T: Fn(&Path, &Path) -> std::io::Result<R>,
    R: Eq + Hash,
    F: Fn(&mut C, R),
    C: Default,
{
    let mut collector = C::default();
    inner_io_task_to(from.as_ref(), to.as_ref(), task, fold, &mut collector)?;
    Ok(collector)
}

fn inner_io_task_to<T, R, F, C>(
    from: &Path,
    to: &Path,
    task: &T,
    fold: &F,
    collector: &mut C,
) -> Result<()>
where
    T: Fn(&Path, &Path) -> std::io::Result<R>,
    R: Eq + Hash,
    F: Fn(&mut C, R),
    C: Default,
{
    let contents = from.read_dir()?;
    contents
        .map(|res| match res {
            Ok(entry) => inner_io_task_into(&entry.path(), to, task, fold, collector),
            Err(e) => Err(e.into()),
        })
        .collect::<Result<()>>()
}

fn symlink(original: &Path, link: &Path) -> std::io::Result<PathBuf> {
    unix::fs::symlink(original, link)?;
    Ok(link.to_owned())
}

fn fold_hash_set_path_buf(collector: &mut HashSet<PathBuf>, path_buf: PathBuf) {
    collector.insert(path_buf);
}

/// Creates links inside the destination corresponding to the file structure of the origin
/// Returns a list of all links created
pub fn link_into<P: AsRef<Path>, Q: AsRef<Path>>(from: P, into: Q) -> Result<HashSet<PathBuf>> {
    io_task_into(from, into, &symlink, &fold_hash_set_path_buf)
}

/// Links all files in source to the destination
/// Returns a list of all links created
pub fn link_to<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<HashSet<PathBuf>> {
    io_task_to(from, to, &symlink, &fold_hash_set_path_buf)
}

/// Links one component paths into another
/// Creates the directories of the destination if they do not exist
/// Returns a list of all links created
pub fn link_component_paths(
    from: &ComponentPaths,
    to: &ComponentPaths,
) -> Result<HashSet<PathBuf>> {
    to.create_dirs()?;
    let mut collector = HashSet::new();

    inner_io_task_to(
        &from.binary,
        &to.binary,
        &symlink,
        &fold_hash_set_path_buf,
        &mut collector,
    )?;
    inner_io_task_to(
        &from.config,
        &to.config,
        &symlink,
        &fold_hash_set_path_buf,
        &mut collector,
    )?;
    inner_io_task_to(
        &from.library,
        &to.library,
        &symlink,
        &fold_hash_set_path_buf,
        &mut collector,
    )?;
    inner_io_task_to(
        &from.share,
        &to.share,
        &symlink,
        &fold_hash_set_path_buf,
        &mut collector,
    )?;

    Ok(collector)
}

/// Links components to component paths.
/// Creates the directories of the destination if they do not exist
/// Returns a list of all links created
pub fn link_components<'a, P: AsRef<Path>>(
    base: P,
    from: impl IntoIterator<Item = &'a Component>,
    to: &'a ComponentPaths,
) -> Result<HashSet<PathBuf>> {
    let base = base.as_ref();

    to.create_dirs()?;
    let mut collector = HashSet::new();

    for component in from {
        let to = match component {
            Component::Binary(_) => &to.binary,
            Component::Config(_) => &to.config,
            Component::Library(_) => &to.library,
            Component::Share(_) => &to.share,
        };
        inner_io_task_into(
            &component.relative_path().to_path(base),
            to,
            &symlink,
            &fold_hash_set_path_buf,
            &mut collector,
        )?;
    }

    Ok(collector)
}

fn copy(from: &Path, to: &Path) -> std::io::Result<u64> {
    fs::copy(from, to)
}

fn fold_u64(collector: &mut u64, val: u64) {
    *collector += val;
}

/// Copies the origin inside the destination
pub fn copy_into<P: AsRef<Path>, Q: AsRef<Path>>(from: P, into: Q) -> Result<u64> {
    io_task_into(from, into, &copy, &fold_u64)
}

/// Copies the content of the origin inside the destination
pub fn copy_to<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<u64> {
    io_task_to(from, to, &copy, &fold_u64)
}
