use super::{object::Objects, package::Packages, *};
use crate::{extra::persist::Pot, GID, UID};
use rustbreak::PathDatabase;
use snafu::ResultExt;
use std::{
    fs,
    os::unix::{self, prelude::PermissionsExt},
    path::PathBuf,
};

#[derive(Debug)]
pub struct LocalBackend {
    path: PathBuf,
    db: PathDatabase<(Objects, Packages), Pot>,
    objects: Objects,
    packages: Packages,
}

impl ReadBackend for LocalBackend {
    type Source = PathBuf;

    fn open(path: PathBuf) -> StoreResult<Self> {
        let db = PathDatabase::load_from_path(path.clone()).context(RustbreakLoadSnafu)?;
        let (objects, packages) = db.get_data(false).context(RustbreakLoadDataSnafu)?;

        Ok(Self {
            path,
            db,
            objects,
            packages,
        })
    }

    fn objects(&self) -> &Objects {
        &self.objects
    }

    fn packages(&self) -> &Packages {
        &self.packages
    }
}

impl WriteBackend for LocalBackend {
    type Source = PathBuf;

    fn init(path: PathBuf) -> StoreResult<Self> {
        let db = PathDatabase::create_at_path(path.clone(), (Objects::new(), Packages::new()))
            .context(RustbreakCreateSnafu)?;

        let mut perm = fs::metadata(&path).context(IoSnafu)?.permissions();
        perm.set_mode(0o644);
        fs::set_permissions(&path, perm).context(IoSnafu)?;
        unix::fs::chown(&path, UID, GID).context(IoSnafu)?;

        Ok(Self {
            path,
            db,
            objects: Objects::new(),
            packages: Packages::new(),
        })
    }

    fn objects_mut(&mut self) -> &mut Objects {
        &mut self.objects
    }

    fn packages_mut(&mut self) -> &mut Packages {
        &mut self.packages
    }

    fn flush(self) -> StoreResult<()> {
        self.db
            .put_data((self.objects, self.packages), true)
            .context(RustbreakSaveDataSnafu)?;
        unix::fs::chown(self.path, UID, GID).context(IoSnafu)?;

        Ok(())
    }
}
