#![feature(let_chains)]
#![feature(slice_pattern)]
#![feature(assert_matches)]
#![feature(vec_retain_mut)]
#![feature(explicit_generic_args_with_impl_trait)]
#![feature(c_size_t)]

// mod cache;
pub mod dependency;
// mod error;
pub mod c_ffi;
pub mod extra;
mod generation;
mod jail;
mod recipe;
mod store;
mod support;
mod user;

// pub use cache::CacheClient;
pub use dependency::{DependencyGraph, Requirement};
// pub use error::Error;
pub use cached_path::{Cache, CacheBuilder};
pub use generation::*;
pub use recipe::Recipe;
pub use semver::{Version, VersionReq};
pub use store::{Blob, LocalBackend, Package, Store};
pub use user::{User, UserManager};

pub const STORE_PATH: &str = "/hua/store/";
