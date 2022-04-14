#![feature(let_chains)]
#![feature(slice_pattern)]
#![feature(assert_matches)]
#![feature(explicit_generic_args_with_impl_trait)]
#![feature(c_size_t)]

pub mod c_ffi;
pub mod dependency;
pub mod extra;
pub mod generation;
pub mod jail;
pub mod recipe;
pub mod shell;
pub mod store;
mod support;
pub mod user;

pub mod cache {
    pub use cached_path::{Cache, CacheBuilder};
}

pub mod version {
    pub use semver::{Version, VersionReq};
}
