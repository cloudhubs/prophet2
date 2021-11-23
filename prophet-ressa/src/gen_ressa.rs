use prophet_ressa_minify::try_minify_ressa;
use source_code_parser::ressa::NodePattern;
use source_code_parser::{ressa::Indexable, Language, ModuleComponent};
use std::collections::HashSet;
use std::fs::DirEntry;

use crate::Error;

pub fn extract_ressas(
    ast: &Vec<ModuleComponent>,
    ressa_dir: &str,
) -> Result<Vec<NodePattern>, Error> {
    // Get subdirectories
    let dirs = get_subdirs(ressa_dir)?;

    // Find languages
    let langs: HashSet<Language> = ast
        .iter()
        .flat_map(|module| find_languages(module as &dyn Indexable))
        .collect();

    // Create ReSSAs for the languages in the project
    let mut ressas = vec![];
    let minified_ressas = dirs
        .iter()
        .flat_map(|entry| {
            if let Some(name) = entry.file_name().to_str() {
                if let Some(lang) = to_lang(name) {
                    Some((name, entry.path().is_dir(), lang))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .flat_map(|(name, is_dir, lang)| {
            if langs.contains(&lang) {
                if is_dir {
                    Some(try_minify_ressa(name))
                } else {
                    tracing::warn!("{:?} not a directory, cannot add {:?} ReSSA", name, lang);
                    None
                }
            } else {
                None
            }
        });

    // Verify no errors, abort if error
    for entry in minified_ressas {
        match entry {
            Ok(ressa) => ressas.push(ressa),
            Err(err) => return Err(Error::MinifyError(err)),
        }
    }

    // Flatten into one vector and return
    Ok(ressas.into_iter().flat_map(|x| x).collect())
}

fn get_subdirs(ressa_dir: &str) -> Result<Vec<DirEntry>, Error> {
    // Validate can check provided directory
    let read_dir = match std::fs::read_dir(&ressa_dir) {
        Ok(dir) => dir,
        Err(err) => {
            tracing::warn!("Could not read directory: {:?}", err);
            return Err(Error::IOError(err));
        }
    };

    // Parse and return subdirectories
    let mut dirs = vec![];
    for dir in read_dir {
        match dir {
            Ok(entry) => dirs.push(entry),
            Err(err) => return Err(Error::IOError(err)),
        }
    }
    Ok(dirs)
}

/// Extract the set of all languages from the LAAST
fn find_languages(ast: &dyn Indexable) -> HashSet<Language> {
    let mut langs: HashSet<Language> = ast
        .get_children()
        .into_iter()
        .map(|child| find_languages(child))
        .flat_map(|langs| langs)
        .collect();
    langs.insert(ast.get_language());
    langs
}

// Convert an optional string into an enum describing the language it lists (None being unknown)
fn to_lang(string: &str) -> Option<Language> {
    match string {
        "cpp" => Some(Language::Cpp),
        "java" => Some(Language::Java),
        "python" => Some(Language::Python),
        "go" => Some(Language::Go),
        _ => None,
    }
}
