use super::{id::PackageId, package::PackageDesc, LocalStore, RemoteStore, StoreResult};
use url::Url;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Source {
    Local,
    Remote { url: Url },
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

    pub fn filter_by_name_containing<'a>(
        &'a self,
        slice: &'a str,
        local: &'a LocalStore,
    ) -> impl Iterator<Item = (&'a PackageId, &'a PackageDesc, Source)> + 'a {
        let remote_iter = self
            .remotes
            .iter()
            .map(move |remote| {
                remote
                    .packages()
                    .filter(move |_id, desc, _set| desc.name.contains(slice))
                    .map(|(id, desc, set)| {
                        let url = unsafe {
                            remote
                                .packages()
                                .url_in_store(id, remote.url())
                                .unwrap_unchecked()
                        };
                        let objects = remote
                            .objects()
                            .get_multiple(set.iter())
                            .filter_map(|(_id, object)| {
                                if object.is_blob() {
                                    Some(object.to_url(&url))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        (id, desc, Source::Remote { base: url, objects })
                    })
            })
            .flatten();

        local
            .packages()
            .filter(move |_id, desc, _set| desc.name.contains(slice))
            .map(|(id, desc, _set)| (id, desc, Source::Local))
            .chain(remote_iter)
    }

    // pub fn find<'a, P>(
    //     &'a self,
    //     predicate: P,
    //     local: &'a LocalStore,
    // ) -> Option<(&'a PackageId, &'a PackageDesc, Source)>
    // where
    //     P: Fn(&PackageId, &PackageDesc, &HashSet<ObjectId>) -> bool,
    // {
    //     if let Some((id, desc)) = local.packages().find(&predicate) {
    //         return Some((id, desc, Source::Local));
    //     }

    //     for remote in self.remotes.iter() {
    //         if let Some((id, desc)) = remote.packages().find(&predicate) {
    //             let url = unsafe {
    //                 remote
    //                     .packages()
    //                     .url_in_store(id, remote.url())
    //                     .unwrap_unchecked()
    //             };
    //             return Some((id, desc, Source::Remote(url)));
    //         }
    //     }

    //     return None;
    // }

    // pub fn find_by_name_containing<'a>(
    //     &'a self,
    //     slice: &str,
    //     local: &'a LocalStore,
    // ) -> Option<(&'a PackageId, &'a PackageDesc, Source)> {
    //     self.find(|_id, desc, _set| desc.name.contains(slice), local)
    // }

    pub fn get_url(&self, index: usize) -> Option<&Url> {
        self.remotes.get(index).map(|store| store.url())
    }
}
