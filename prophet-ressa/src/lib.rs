mod gen_ressa;

use std::path::Path;

use gen_ressa::*;
use source_code_parser::{
    ressa::{run_ressa_parse, RessaResult},
    ModuleComponent,
};

/// Errors that arise while running the ReSSA
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(String),
    #[error("Minify Error: {0}")]
    Minify(String),
}

impl From<std::io::Error> for Error {
    fn from(io_error: std::io::Error) -> Self {
        Error::Io(io_error.to_string())
    }
}

impl From<prophet_ressa_minify::MinifyError> for Error {
    fn from(min_error: prophet_ressa_minify::MinifyError) -> Self {
        Error::Minify(min_error.to_string())
    }
}

/// Run ressas in the described directory against the provided LAAST
pub fn run_ressa(ast: &mut Vec<ModuleComponent>, ressa_dir: &Path) -> Result<RessaResult, Error> {
    let ressas = extract_ressas(ast, ressa_dir)?;
    Ok(run_ressa_parse(ast, ressas))
}
