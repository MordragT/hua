use url::Url;

use super::{
    backend::ReadBackend, id::PackageId, package::PackageDesc, RemoteStore, Store, StoreResult,
};

pub enum Source {
    Local,
    Remote(usize),
}

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

    pub fn find_by_name<'a, S, B: ReadBackend>(
        &'a self,
        name: &str,
        local: &'a Store<S, B>,
    ) -> Option<(&'a PackageId, &'a PackageDesc, Source)> {
        if let Some((id, desc)) = local.packages().find_by_name(name) {
            return Some((id, desc, Source::Local));
        }

        for (n, remote) in self.remotes.iter().enumerate() {
            if let Some((id, desc)) = remote.packages().find_by_name(name) {
                return Some((id, desc, Source::Remote(n)));
            }
        }

        return None;
    }

    pub fn get_url(&self, index: usize) -> Option<&Url> {
        self.remotes.get(index).map(|store| store.url())
    }
}
