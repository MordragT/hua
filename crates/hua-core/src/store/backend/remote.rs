use cached_path::CacheBuilder;
use rustbreak::PathDatabase;
use snafu::ResultExt;
use url::Url;

use super::{object::Objects, package::Packages, *};

use crate::extra::persist::Pot;

use super::ReadBackend;

/// A purely immutable [ReadBackend] for remote access of packages.
#[derive(Debug)]
pub struct RemoteBackend {
    objects: Objects,
    packages: Packages,
}

impl ReadBackend for RemoteBackend {
    type Source = Url;

    fn open(source: Self::Source) -> crate::store::StoreResult<Self> {
        let cache = CacheBuilder::default().build().context(CacheSnafu)?;

        let path = cache.cached_path(source.as_str()).context(CacheSnafu)?;

        let db = PathDatabase::<(Objects, Packages), Pot>::load_from_path(path)
            .context(RustbreakLoadSnafu)?;
        let (objects, packages) = db.get_data(false).context(RustbreakLoadDataSnafu)?;

        Ok(Self { objects, packages })
    }

    fn packages(&self) -> &crate::store::package::Packages {
        &self.packages
    }

    fn objects(&self) -> &crate::store::object::Objects {
        &self.objects
    }
}
