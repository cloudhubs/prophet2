use prophet_ressa_minify::try_minify_ressa;
use source_code_parser::ressa::NodePattern;
use source_code_parser::{ressa::Indexable, Language, ModuleComponent};
use std::collections::HashSet;
use std::fs::DirEntry;
use std::path::Path;

use crate::Error;

/// Use the langages in the provided LAAST to determine and load the needed ReSSAs
pub fn extract_ressas(
    ast: &[ModuleComponent],
    ressa_dir: &Path,
) -> Result<Vec<NodePattern>, Error> {
    // Get subdirectories
    let dirs = get_subdirs(ressa_dir)?;

    // Find languages
    let langs = ast
        .iter()
        .flat_map(|module| find_languages(module as &dyn Indexable))
        .collect::<HashSet<Language>>();

    // Create ReSSAs for the languages in the project
    let mut ressas = vec![];
    let minified_ressas = dirs
        .into_iter()
        .flat_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            let lang = to_lang(name)?;
            Some((name.to_string(), entry.path().is_dir(), lang))
        })
        .flat_map(|(name, is_dir, lang)| {
            if langs.contains(&lang) && is_dir {
                return Some(try_minify_ressa(name));
            } else if !is_dir {
                tracing::warn!("{:?} not a directory, cannot add {:?} ReSSA", name, lang);
            }
            None
        });

    // Verify no errors, abort if error
    for entry in minified_ressas {
        let ressa = entry.map_err::<Error, _>(|err| err.into())?;
        ressas.push(ressa);
    }

    // Flatten into one vector and return
    Ok(ressas.into_iter().flatten().collect())
}

/// Retrieve the subdirectories of the directory named by the provided string
fn get_subdirs(ressa_dir: &Path) -> Result<Vec<DirEntry>, Error> {
    // Validate can check provided directory
    let read_dir = std::fs::read_dir(ressa_dir).map_err(|err| Error::Io(err.to_string()))?;

    // Parse and return subdirectories
    let mut dirs = vec![];
    for dir in read_dir {
        let entry = dir.map_err::<Error, _>(|err| err.into())?;
        dirs.push(entry);
    }
    Ok(dirs)
}

/// Extract the set of all languages from the LAAST
fn find_languages(ast: &dyn Indexable) -> HashSet<Language> {
    let mut langs: HashSet<Language> = ast
        .get_children()
        .into_iter()
        .flat_map(|child| find_languages(child))
        .collect();
    langs.insert(ast.get_language());
    langs
}

// Convert to an enum describing the language the string describes
/// (unknown being coerced to None)
fn to_lang(string: &str) -> Option<Language> {
    match string.to_string().into() {
        Language::Unknown => None,
        string => Some(string),
    }
}
