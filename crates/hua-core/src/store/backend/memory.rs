use super::{object::Objects, package::Packages, *};
use std::any::Any;

#[derive(Debug)]
pub struct MemoryBackend {
    objects: Objects,
    packages: Packages,
    derivations: Derivations,
}

impl ReadBackend for MemoryBackend {
    type Source = Box<dyn Any>;

    fn open(_source: Self::Source) -> StoreResult<Self> {
        panic!("The memory backend is just for testing cannot open")
    }

    fn objects(&self) -> &Objects {
        &self.objects
    }

    fn packages(&self) -> &Packages {
        &self.packages
    }

    fn derivations(&self) -> &Derivations {
        &self.derivations
    }
}

impl WriteBackend for MemoryBackend {
    type Source = Box<dyn Any>;

    fn init(_source: Self::Source) -> StoreResult<Self> {
        Ok(Self {
            objects: Objects::new(),
            packages: Packages::new(),
            derivations: Derivations::new(),
        })
    }

    fn objects_mut(&mut self) -> &mut Objects {
        &mut self.objects
    }

    fn packages_mut(&mut self) -> &mut Packages {
        &mut self.packages
    }

    fn derivations_mut(&mut self) -> &mut Derivations {
        &mut self.derivations
    }

    fn flush(self) -> StoreResult<()> {
        Ok(())
    }
}
