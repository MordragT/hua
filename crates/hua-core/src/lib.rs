#![feature(let_chains)]
#![feature(slice_pattern)]
#![feature(assert_matches)]

// mod cache;
pub mod dependency;
mod download;
mod error;
pub mod extra;
mod generation;
mod recipe;
mod store;
mod support;
mod user;

// pub use cache::CacheClient;
pub use dependency::{DependencyGraph, Requirement};
pub use download::Downloader;
pub use error::Error;
pub use generation::*;
pub use recipe::Recipe;
pub use semver::{Version, VersionReq};
pub use store::{Package, Store};
pub use user::{User, UserManager};
