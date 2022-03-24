use std::path::PathBuf;

use crate::{error::Result, extra::Source};

/// An Downloader to retrieve files from different sources
#[derive(Debug)]
pub struct Downloader {}

impl Downloader {
    /// Downloads data if it is not local and verifies them
    pub fn download(&mut self, source: &Source) -> Result<PathBuf> {
        todo!()
    }

    /// Downloads multiple data sources if not local and verifes them
    pub fn batch_download(&mut self, sources: &[Source]) -> Result<Vec<PathBuf>> {
        todo!()
    }
}
