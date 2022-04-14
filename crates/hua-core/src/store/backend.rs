use super::{object::Objects, package::Packages, *};
use crate::extra::persist::Pot;
use rustbreak::PathDatabase;
use snafu::ResultExt;
use std::{any::Any, path::PathBuf};

pub trait Backend: Sized + std::fmt::Debug {
    type Source;

    fn init(source: Self::Source) -> StoreResult<Self>;
    fn open(source: Self::Source) -> StoreResult<Self>;
    fn objects(&self) -> &Objects;
    fn objects_mut(&mut self) -> &mut Objects;
    fn packages(&self) -> &Packages;
    fn packages_mut(&mut self) -> &mut Packages;
    fn flush(self) -> StoreResult<()>;
}

#[derive(Debug)]
pub struct LocalBackend {
    db: PathDatabase<(Objects, Packages), Pot>,
    objects: Objects,
    packages: Packages,
}

impl Backend for LocalBackend {
    type Source = PathBuf;

    fn init(path: PathBuf) -> StoreResult<Self> {
        let db = PathDatabase::create_at_path(path, (Objects::new(), Packages::new()))
            .context(RustbreakCreateSnafu)?;

        Ok(Self {
            db,
            objects: Objects::new(),
            packages: Packages::new(),
        })
    }

    fn open(path: PathBuf) -> StoreResult<Self> {
        let db = PathDatabase::load_from_path(path).context(RustbreakLoadSnafu)?;

        let (objects, packages) = db.get_data(false).context(RustbreakLoadDataSnafu)?;

        Ok(Self {
            db,
            objects,
            packages,
        })
    }

    fn objects(&self) -> &Objects {
        &self.objects
    }

    fn objects_mut(&mut self) -> &mut Objects {
        &mut self.objects
    }

    fn packages(&self) -> &Packages {
        &self.packages
    }

    fn packages_mut(&mut self) -> &mut Packages {
        &mut self.packages
    }

    fn flush(self) -> StoreResult<()> {
        self.db
            .put_data((self.objects, self.packages), true)
            .context(RustbreakSaveDataSnafu)?;
        Ok(())
    }
}

#[derive(Debug)]
struct MemoryBackend {
    objects: Objects,
    packages: Packages,
}

impl Backend for MemoryBackend {
    type Source = Box<dyn Any>;

    fn init(_source: Self::Source) -> StoreResult<Self> {
        Ok(Self {
            objects: Objects::new(),
            packages: Packages::new(),
        })
    }

    fn open(_source: Self::Source) -> StoreResult<Self> {
        panic!("The memory backend is just for testing cannot open")
    }

    fn objects(&self) -> &Objects {
        &self.objects
    }

    fn objects_mut(&mut self) -> &mut Objects {
        &mut self.objects
    }

    fn packages(&self) -> &Packages {
        &self.packages
    }

    fn packages_mut(&mut self) -> &mut Packages {
        &mut self.packages
    }

    fn flush(self) -> StoreResult<()> {
        Ok(())
    }
}
