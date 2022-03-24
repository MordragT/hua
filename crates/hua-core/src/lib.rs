#![feature(let_chains)]
#![feature(slice_pattern)]

mod cache;
mod components;
mod dependency;
mod download;
mod error;
pub mod extra;
mod generation;
mod package;
mod persist;
mod recipe;
mod store;
mod user;

pub use cache::CacheClient;
pub use components::*;
pub use dependency::Requirement;
pub use download::Downloader;
pub use error::Error;
pub use generation::*;
pub use package::Package;
pub use recipe::Recipe;
pub use semver::{Version, VersionReq};
pub use store::Store;
pub use user::{User, UserManager};
