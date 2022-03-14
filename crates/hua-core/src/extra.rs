use crate::{error::*, ComponentPaths, OptionalComponentPaths};
use std::{
    fs,
    hash::{Hash, Hasher},
    os::unix,
    path::Path,
};

// TODO: better naming

pub fn io_operation_into<P, Q, F>(from: P, into: Q, operation: &F) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    F: Fn(&Path, &Path) -> std::io::Result<()>,
{
    let from = from.as_ref().canonicalize()?;
    let name = from
        .file_name()
        .ok_or(Error::TerminatingPath)?
        .to_str()
        .ok_or(Error::OsStringConversion)?;
    let into = into.as_ref().join(name);

    if from.is_dir() {
        from.read_dir()?
            .map(|res| match res {
                Ok(entry) => {
                    if !into.exists() {
                        fs::create_dir(&into)?;
                    }
                    io_operation_into(entry.path(), &into, operation)
                }
                Err(e) => Err(e.into()),
            })
            .collect::<Result<()>>()
    } else {
        operation(&from, &into)?;
        Ok(())
    }
}

pub fn io_operation_to<P, Q, F>(from: P, to: Q, operation: &F) -> Result<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
    F: Fn(&Path, &Path) -> std::io::Result<()>,
{
    let from = from.as_ref();
    let contents = from.read_dir()?;
    contents
        .map(|res| match res {
            Ok(entry) => io_operation_into(entry.path(), to.as_ref(), operation),
            Err(e) => Err(e.into()),
        })
        .collect::<Result<()>>()
}

fn symlink(original: &Path, link: &Path) -> std::io::Result<()> {
    unix::fs::symlink(original, link)
}

/// Creates links inside the destination corresponding to the file structure of the origin
pub fn link_into<P: AsRef<Path>, Q: AsRef<Path>>(from: P, into: Q) -> Result<()> {
    io_operation_into(from, into, &symlink)
}

pub fn link_to<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    io_operation_to(from, to, &symlink)
}

/// Links one component paths into another
/// Creates the directories of the destination if they do not exist
pub fn link_component_paths(from: &ComponentPaths, to: &ComponentPaths) -> Result<()> {
    to.create_dirs()?;

    link_to(&from.binary, &to.binary)?;
    link_to(&from.config, &to.config)?;
    link_to(&from.library, &to.library)?;
    link_to(&from.share, &to.share)?;
    Ok(())
}

/// Links optional component paths to normal component paths
/// Creates the directories of the destination if they do not exist
pub fn link_optional_component_paths(
    from: &OptionalComponentPaths,
    to: &ComponentPaths,
) -> Result<()> {
    to.create_dirs()?;

    if let Some(p) = &from.binary {
        link_to(&p, &to.binary)?;
    }
    if let Some(p) = &from.config {
        link_to(&p, &to.config)?;
    }
    if let Some(p) = &from.library {
        link_to(&p, &to.library)?;
    }
    if let Some(p) = &from.share {
        link_to(&p, &to.share)?;
    }
    Ok(())
}

fn copy(from: &Path, to: &Path) -> std::io::Result<()> {
    let _num = fs::copy(from, to)?;
    Ok(())
}

/// Copies the origin inside the destination
pub fn copy_into<P: AsRef<Path>, Q: AsRef<Path>>(from: P, into: Q) -> Result<()> {
    io_operation_into(from, into, &copy)
}

/// Copies the content of the origin inside the destination
pub fn copy_to<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Result<()> {
    io_operation_to(from, to, &copy)
}

pub fn hash_path<P: AsRef<Path>, H: Hasher>(path: P, state: &mut H) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        path.read_dir()?
            .map(|res| match res {
                Ok(entry) => hash_path(entry.path(), state),
                Err(e) => Err(e.into()),
            })
            .collect::<Result<()>>()
    } else {
        fs::read(path)?.hash(state);
        Ok(())
    }
}
