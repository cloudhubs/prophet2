//! A library for cloning microservice git repositories and
use std::path::{Path, PathBuf};

use git2::Repository;
use serde::Deserialize;
use source_code_parser::Directory;

/// A cloned microservice or microservice system repository
#[derive(Debug, Default, Deserialize)]
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

/// Gets the subdirectories and files from a directory `&Path`
fn get_dir_contents(root_dir: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), std::io::Error> {
    let read_dir = match std::fs::read_dir(&root_dir) {
        Ok(dir) => dir,
        Err(err) => {
            tracing::warn!("Could not read directory: {:?}", err);
            return Err(err);
        }
    };

    let mut files = vec![];
    let mut sub_dirs = vec![];

    for entry in read_dir {
        let entry = entry?;
        let entry_path = entry.path();

        if entry_path.is_dir() {
            sub_dirs.push(entry_path);
            continue;
        }

        files.push(entry_path);
    }

    Ok((files, sub_dirs))
}

/// Convers the given root directory into its Directory representation
fn convert_sub_dir(root_dir: PathBuf) -> Option<Directory> {
    let (files, sub_dir_paths) = match get_dir_contents(&root_dir) {
        Ok(contents) => contents,
        _ => return None,
    };

    let sub_directories = convert_sub_dirs(sub_dir_paths);

    Some(Directory::new(files, sub_directories, root_dir))
}

/// Converts the paths into their Directory representations
fn convert_sub_dirs(sub_dirs: Vec<PathBuf>) -> Vec<Directory> {
    sub_dirs.into_iter().flat_map(convert_sub_dir).collect()
}

impl From<MicroservicesRepository> for Directory {
    /// Create a Directory structure from a cloned microservice(s) repository
    fn from(repo: MicroservicesRepository) -> Self {
        // Convert into the Directory type from the given MS repository
        let root_dirs = repo
            .root_dirs
            .iter()
            .map(|relative_path| {
                let mut root = repo.clone_dir.clone();
                root.push(relative_path);
                root
            })
            .flat_map(convert_sub_dir)
            .collect();

        // Get the files in the root directory
        let files = match get_dir_contents(&repo.clone_dir.clone()) {
            Ok((files, _)) => files,
            _ => vec![],
        };

        Directory::new(files, root_dirs, repo.clone_dir.clone())
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
#[derive(Debug, Deserialize)]
pub struct Repositories(Vec<MicroservicesRepository>);

impl From<Repositories> for Directory {
    /// Create a Directory structure from cloned microservice repositories
    fn from(repositories: Repositories) -> Self {
        // Convert into the Directory type from the given
        // repositories and root directories for each
        let sub_directories = repositories
            .0
            .into_iter()
            .map(MicroservicesRepository::into)
            .collect::<Vec<Directory>>();

        // Create a fake top-level directory
        Directory::new(vec![], sub_directories, "".into())
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
