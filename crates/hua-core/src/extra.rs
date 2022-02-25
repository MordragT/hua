use crate::error::*;
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
