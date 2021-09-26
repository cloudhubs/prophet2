//! A library for constructing a ReSSA with minified Rune script files
use std::{fs::File, io::Read, path::Path};

use source_code_parser::ressa::NodePattern;
use thiserror::Error;

/// This type represents all possible errors that can occur when creating
/// a ReSSA from its file parts
#[derive(Debug, Clone, Error)]
pub enum MinifyError {
    #[error("Could not read file: {0}")]
    Io(String),
    #[error("Could not deserialize ReSSA JSON: {0}")]
    Deserialize(String),
}

impl From<std::io::Error> for MinifyError {
    fn from(io_error: std::io::Error) -> Self {
        MinifyError::Io(io_error.to_string())
    }
}

impl From<serde_json::Error> for MinifyError {
    fn from(de_error: serde_json::Error) -> Self {
        MinifyError::Deserialize(de_error.to_string())
    }
}

/// Creates a minified ReSSA from the provided ReSSA path
pub fn try_minify_ressa<P: AsRef<Path>>(path: P) -> Result<Vec<NodePattern>, MinifyError> {
    // Deserialize
    let ressa_file: File = File::open(path.as_ref().to_path_buf())?;

    let mut ressa: Vec<NodePattern> = serde_json::from_reader(ressa_file)?;
    let mut base_path = path.as_ref().to_path_buf();
    base_path.pop();

    // Minify and replace
    for pat in ressa.iter_mut() {
        minify_ressa_script(pat, &base_path)?;
    }

    Ok(ressa)
}

fn minify_ressa_script(pat: &mut NodePattern, base_path: &Path) -> Result<(), MinifyError> {
    // Minify script if there is one
    if let Some(callback_path) = pat.callback.as_mut() {
        let mut path = base_path.to_path_buf();
        path.push(&callback_path);
        let mut script_file: File = File::open(path)?;
        let mut script = String::new();

        script_file.read_to_string(&mut script)?;

        *callback_path = script
            .replace("    ", "")
            .replace("  ", "")
            .replace("\n", "")
            .replace("\r", "");
    }

    // Minify subpattern scripts
    for subpattern in pat.subpatterns.iter_mut() {
        minify_ressa_script(subpattern, base_path)?
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minify_simple() {
        let ressa =
            try_minify_ressa("res/deathstarbench/simple/ressa.json").expect("Failed to minify");
        let expected = r#"let endpoint = ctx.get_variable("endpoint").unwrap();let service = ctx.get_variable("service_name").unwrap();if (!endpoint.ends_with("Handler")) {ctx.make_attribute(service, endpoint, None);}"#;
        assert_eq!(
            Some(expected.to_string()),
            ressa.get(0).unwrap().subpatterns.get(0).unwrap().callback
        );
    }
}
