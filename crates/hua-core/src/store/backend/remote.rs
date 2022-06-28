use cached_path::CacheBuilder;
use rustbreak::PathDatabase;
use snafu::ResultExt;
use url::Url;

use super::{object::Objects, package::Packages, *};

use crate::extra::persist::Pot;

use super::ReadBackend;

#[derive(Debug)]
pub struct RemoteBackend {
    objects: Objects,
    packages: Packages,
    derivations: Derivations,
}

impl ReadBackend for RemoteBackend {
    type Source = Url;

    fn open(source: Self::Source) -> crate::store::StoreResult<Self> {
        let cache = CacheBuilder::default().build().context(CacheSnafu)?;

        let path = cache.cached_path(source.as_str()).context(CacheSnafu)?;

        let db = PathDatabase::<(Objects, Packages, Derivations), Pot>::load_from_path(path)
            .context(RustbreakLoadSnafu)?;
        let (objects, packages, derivations) =
            db.get_data(false).context(RustbreakLoadDataSnafu)?;

        Ok(Self {
            objects,
            packages,
            derivations,
        })
    }

    fn packages(&self) -> &Packages {
        &self.packages
    }

    fn objects(&self) -> &Objects {
        &self.objects
    }

    fn derivations(&self) -> &Derivations {
        &self.derivations
    }
}
