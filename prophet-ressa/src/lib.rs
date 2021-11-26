mod gen_ressa;

use gen_ressa::*;
use source_code_parser::{ModuleComponent, ressa::{RessaResult, run_ressa_parse}};

/// Errors that arise while retrieving entity/endpoint pairs
#[ derive() ]
pub enum Error {
    IOError(std::io::Error),
    MinifyError(prophet_ressa_minify::MinifyError),
}

/// Generate a string for displaying the error
impl From<Error> for String {
    fn from(err: Error) -> Self {
        match err {
            Error::IOError(io_err) => format!("IO error: {}", io_err),
            Error::MinifyError(min_err) => format!("Minify error: {}", min_err),
        }
    }
}

/// Run ressas in the described directory against the provided LAAST
pub fn run_ressa(
    ast: &mut Vec<ModuleComponent>,
    ressa_dir: &str,
) -> Result<RessaResult, Error> {
    let ressas = extract_ressas(ast, ressa_dir)?;
    Ok(run_ressa_parse(ast, ressas))
}
