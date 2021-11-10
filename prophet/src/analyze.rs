use super::*;
use source_code_parser::{parse_project_context, ressa::run_ressa_parse, Directory};

// TODO determine the form of this
pub struct AppData {}

/// This type represents all possible errors that can occur when analyzing
/// a microservice project
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Could not clone repository: {0}")]
    CloneRepo(String),
    #[error("Encountered an IO error: {0}")]
    Io(String),
}

macro_rules! error_from_impl {
    ( $( $error_type:path: $variant:ident ),+ ) => {
        $(
            impl From<$error_type> for Error {
                fn from(error: $error_type) -> Self {
                    Error::$variant(error.to_string())
                }
            }
        )*
    };
}

error_from_impl!(git2::Error: CloneRepo, std::io::Error: Io);

/// Clone the provided repositories and generate ReSSAs to analyze them
/// based on the languages in its LAAST
pub fn get_app_data(mut repos: Repositories) -> Result<AppData, Error> {
    repos.clone_all()?;

    let dir: Directory = repos.into();
    let mut laast = parse_project_context(&dir)?;
    // Generate ReSSAs based on languages in ctx modules
    let ressas = vec![];
    let _result = run_ressa_parse(&mut laast.modules, ressas);

    // TODO determine return type for what makes sense to return here,
    // and then create an adapter for what the frontend of Prophet needs

    Ok(AppData {})
    // Clean up repos on disk on drop
}
