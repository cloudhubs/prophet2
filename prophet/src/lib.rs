//! A library for cloning microservice git repositories and
use std::path::PathBuf;

use git2::Repository;
use serde::Deserialize;

/// A cloned microservice or microservice system repository
#[derive(Default, Deserialize)]
pub struct MicroservicesRepository {
    /// The Git URL of the repository to clone from
    pub git_url: String,
    /// The root directories to include source code files from in static analysis,
    /// relative to `clone_dir`
    pub root_dirs: Vec<PathBuf>,
    /// The local directory the repository was cloned to
    pub clone_dir: PathBuf,
}

impl MicroservicesRepository {
    /// Clones a microservice(s) repository
    pub fn clone(&mut self) -> Result<(), git2::Error> {
        Repository::clone(&self.git_url, &self.clone_dir)?;
        Ok(())
    }
}

impl Drop for MicroservicesRepository {
    /// Clean the cloned repository when freeing the repository from memory
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_dir_all(&self.clone_dir) {
            tracing::warn!(
                "Failed to remove cloned repository '{}' at '{:?}': {:?}",
                self.git_url,
                self.clone_dir,
                err
            );
        }
    }
}

/// The cloned repositories for the microservices to statically analyze
///
/// The serialized representation in JSON is as follows
/// ```json
/// [
///   {
///      "git_url": "https://github.com/some/repository.git",
///      "root_dirs": ["some/relative", "./paths/here"],
///      "clone_dir": "/path/to/the/cloned/repo"
///   }
/// ]
/// ```
#[derive(Deserialize)]
pub struct Repositories(Vec<MicroservicesRepository>);

impl Into<source_code_parser::Directory> for Repositories {
    fn into(self) -> source_code_parser::Directory {
        let (repositories) = self.0;

        // Convert into the Directory type from the given
        // repositories and root directories for each

        // TODO add new() method to this type or make its fields pub
        // source_code_parser::Directory {
        //     files: vec![],
        //     sub_directories: vec![],
        //     path: "".into(),
        // }
        todo!()
    }
}

impl Repositories {
    /// Clones all of the microservice(s) repositories
    pub fn clone_all(&mut self) -> Result<(), git2::Error> {
        for repo in self.0.iter_mut() {
            repo.clone()?;
        }
        Ok(())
    }
}

// TODO implement getProphetAppData equivalent
