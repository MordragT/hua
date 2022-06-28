use super::{object::Objects, package::Packages, *};

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
}

pub trait WriteBackend: Sized + std::fmt::Debug {
    type Source;

    fn init(source: Self::Source) -> StoreResult<Self>;
    fn objects_mut(&mut self) -> &mut Objects;
    fn packages_mut(&mut self) -> &mut Packages;
    fn flush(self) -> StoreResult<()>;
}
