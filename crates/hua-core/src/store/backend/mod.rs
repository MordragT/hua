use super::{derivation::Derivations, object::Objects, package::Packages, *};

pub use local::LocalBackend;
pub use memory::MemoryBackend;
pub use remote::RemoteBackend;

mod local;
mod memory;
mod remote;

pub trait ReadBackend: Sized + std::fmt::Debug {
    type Source;

    fn open(source: Self::Source) -> StoreResult<Self>;
    fn objects(&self) -> &Objects;
    fn packages(&self) -> &Packages;
    fn derivations(&self) -> &Derivations;
}

pub trait WriteBackend: Sized + std::fmt::Debug {
    type Source;

    fn init(source: Self::Source) -> StoreResult<Self>;
    fn objects_mut(&mut self) -> &mut Objects;
    fn packages_mut(&mut self) -> &mut Packages;
    fn derivations_mut(&mut self) -> &mut Derivations;
    fn flush(self) -> StoreResult<()>;
}
