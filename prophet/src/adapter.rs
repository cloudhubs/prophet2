use std::path::Path;

use super::Error;
use serde::Serialize;

use super::Repositories;

/// A compatibility AppData type for the current Prophet frontend
#[derive(Debug, Default, Serialize)]
pub struct AppData {}

impl AppData {
    /// Clone the provided repositories and generate ReSSAs to analyze them
    /// based on the languages in its LAAST
    pub fn from_repositories<P: AsRef<Path>>(repos: Repositories, ressa_dir: P) -> Result<AppData, Error> {
        super::AppData::from_repositories(repos, ressa_dir).map(AppData::from)
    }
}

impl From<super::AppData> for AppData {
    fn from(_app_data: super::AppData) -> Self {
        // TODO
        AppData::default()
    }
}
