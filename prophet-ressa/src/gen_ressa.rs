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
            // Retrieve language the directory represents
            let name = entry.file_name();
            let lang = to_lang(name.to_str()?)?;

            // If it isn't a language in the project, ignore it
            if langs.contains(&lang) {
                Some(entry)
            } else {
                None
            }
        })
        .map(|entry| {
            // Get subdirectories containing expanded ReSSAs
            std::fs::read_dir(entry.path()).map_err(Error::from)
        })
        .collect::<Vec<_>>();

    // Verify no errors, abort if error
    for entry in minified_ressas {
        for ressa_dir in entry? {
            // Get the base ReSSA file from the directory
            let mut ressa_dir = ressa_dir?.path();
            ressa_dir.push("ressa.json");

            // Read in and store the ReSSA
            let mut result_ressa = try_minify_ressa(ressa_dir).map_err(Error::from)?;
            ressas.append(&mut result_ressa);
        }
    }

    // Flatten into one vector and return
    Ok(ressas)
}

/// Retrieve the subdirectories of the directory named by the provided string
fn get_subdirs(ressa_dir: &Path) -> Result<Vec<DirEntry>, Error> {
    // Validate can check provided directory
    let read_dir = std::fs::read_dir(ressa_dir)?;

    // Parse and return subdirectories
    let mut dirs = vec![];
    for dir in read_dir {
        dirs.push(dir?);
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
