use super::{object::Objects, package::Packages, *};

pub use local::LocalBackend;
pub use memory::MemoryBackend;
pub use remote::RemoteBackend;

mod local;
mod memory;
mod remote;

/// A backend for the [Store](crate::store::Store) for purely immutable read operations.
pub trait ReadBackend: Sized + std::fmt::Debug {
    /// The type with wich the backend can be opened.
    type Source;

    /// Opens a [ReadBackend] with the specified source.
    fn open(source: Self::Source) -> StoreResult<Self>;
    /// Borrow [Objects] immutable.
    fn objects(&self) -> &Objects;
    /// Borrow [Packages] immutable.
    fn packages(&self) -> &Packages;
}

/// A backend for the [Store](crate::store::Store) for mutable write operations.
pub trait WriteBackend: Sized + std::fmt::Debug {
    /// The type with wich the backend can be initialised.
    type Source;

    /// Initialise the [WriteBackend] at source.
    fn init(source: Self::Source) -> StoreResult<Self>;
    /// Borrow [Objects] mutable.
    fn objects_mut(&mut self) -> &mut Objects;
    /// Borrow [Packages] mutable.
    fn packages_mut(&mut self) -> &mut Packages;
    /// Flush all changes to the backend.
    fn flush(self) -> StoreResult<()>;
}
