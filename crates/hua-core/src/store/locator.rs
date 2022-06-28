use super::{package::RemotePackageSource, RemoteStore, StoreResult};
use crate::recipe::Derivation;
use url::Url;

#[derive(Debug)]
pub struct Locator {
    remotes: Vec<RemoteStore>,
}

// TODO: open remote stores only if package was not found before so that failing remotes
// do not cause the whole thing to blow up

impl Locator {
    pub fn new(remotes: impl IntoIterator<Item = Url>) -> StoreResult<Self> {
        let remotes = remotes
            .into_iter()
            .map(|url| RemoteStore::open(url))
            .collect::<StoreResult<_>>()?;

        Ok(Self { remotes })
    }

    pub fn search<'a>(
        &'a self,
        drv: &'a Derivation,
    ) -> impl Iterator<Item = RemotePackageSource> + 'a {
        self.remotes
            .iter()
            .map(move |remote| {
                remote
                    .packages()
                    .filter(move |_, other_drv, _| drv == other_drv)
                    .map(|(id, drv, ids)| {
                        let base = remote.url().clone();

                        let blobs = remote.objects().get_blobs_ids_cloned(ids).collect();
                        let trees = remote.objects().get_trees_ids_cloned(ids).collect();
                        RemotePackageSource::new(*id, drv.clone(), base, blobs, trees)
                    })
            })
            .flatten()
    }

    pub fn get_url(&self, index: usize) -> Option<&Url> {
        self.remotes.get(index).map(|store| store.url())
    }
}
