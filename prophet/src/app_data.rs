use std::path::Path;

use crate::{Error, Repositories};
use prophet_ressa::run_ressa;

use prophet_bounded_context::get_bounded_context;
use prophet_mermaid::MermaidString;
use prophet_model::{MicroserviceGraph, MicroserviceEntities};
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
    pub async fn from_ressa_result(ressa_result: &RessaResult) -> Result<AppData, Error> {
        // Retrieve microservices
        let ms_graph = match MicroserviceGraph::try_new(ressa_result) {
            Some(ms_graph) => ms_graph,
            None => return Err(Error::AppData("Could not create microservice graph".into())),
        };
        let microservices = ms_graph.nodes();

        // Retrieve entities
        let entities: Vec<_> = match MicroserviceEntities::try_new(ressa_result) {
            Some(entities) => entities,
            None => return Err(Error::AppData("Could not create entity data".into())),
        }.into();

        // Get the bounded context and its diagram
        let bounded_entity_graph = get_bounded_context(&entities).await?;
        let entity_diagram = Some(MermaidString::from(bounded_entity_graph.clone()));

        // Get the microservice communication diagram
        let communication_diagram = Some(MermaidString::from(ms_graph));

        // Get the microservice bounded entity diagrams
        let microservices = microservices
            .into_iter()
            .map(|ms| {
                let mut entity_graph = bounded_entity_graph.clone();
                entity_graph.filter_entities(&entities);
                Microservice {
                    name: ms.name,
                    entity_diagram: Some(MermaidString::from(entity_graph)),
                }
            })
            .collect();

        Ok(AppData {
            name: "system".into(),
            communication_diagram,
            entity_diagram,
            microservices,
        })
    }

    /// Clone the provided repositories and generate ReSSAs to analyze them
    /// based on the languages in its LAAST
    pub async fn from_repositories<P: AsRef<Path>>(
        mut repos: Repositories,
        ressa_dir: P,
    ) -> Result<AppData, Error> {
        repos.clone_all()?;

        let dir: Directory = repos.into();
        let mut laast = parse_project_context(&dir)?;
        // Generate ReSSAs based on languages in ctx modules
        let result: RessaResult = run_ressa(&mut laast.modules, ressa_dir.as_ref())
            .map_err(|err| Error::AppData(err.to_string()))?;

        AppData::from_ressa_result(&result).await
        // Clean up repos on disk on drop
    }
}
