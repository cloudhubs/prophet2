use std::path::Path;

use crate::{Error, Repositories};
use prophet_ressa::run_ressa;

use prophet_mermaid::MermaidString;
use serde::Serialize;
use source_code_parser::{parse_project_context, ressa::RessaResult, Directory};

/// An analyzed microservice within a project
#[derive(Debug, Default, Serialize)]
pub struct Microservice {
    /// The name of the microservice
    pub name: String,
    /// The entity diagram for the analyzed microservice,
    pub entity_diagram: Option<MermaidString>,
}

/// The analyzed data for the provided project
#[derive(Debug, Default, Serialize)]
pub struct AppData {
    /// The application or project name
    pub name: String,
    /// The microservice or endpoint communication diagram for
    /// the analyzed project
    pub communication_diagram: Option<MermaidString>,
    /// The entity diagram for the analyzed project
    pub entity_diagram: Option<MermaidString>,
    /// The microservices in the analyzed project
    pub microservices: Vec<Microservice>,
}

impl AppData {
    /// Creates an AppData from the results of a ReSSA
    pub fn from_ressa_result(_ressa_result: &RessaResult) -> Result<AppData, Error> {
        // TODO
        Ok(AppData::default())
    }

    /// Clone the provided repositories and generate ReSSAs to analyze them
    /// based on the languages in its LAAST
    pub fn from_repositories<P: AsRef<Path>>(
        mut repos: Repositories,
        ressa_dir: P,
    ) -> Result<AppData, Error> {
        repos.clone_all()?;

        let dir: Directory = repos.into();
        let mut laast = parse_project_context(&dir)?;
        // Generate ReSSAs based on languages in ctx modules
        let result: RessaResult = run_ressa(&mut laast.modules, ressa_dir.as_ref())
            .map_err(|err| Error::AppData(err.to_string()))?;

        AppData::from_ressa_result(&result)
        // Clean up repos on disk on drop
    }
}
