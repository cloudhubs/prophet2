use std::path::Path;

use crate::{Error, Repositories};
use prophet_ressa::run_ressa;

use prophet_bounded_context::get_bounded_context;
use prophet_mermaid::MermaidString;
use prophet_model::MicroserviceGraph;
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
        let ms_graph = match MicroserviceGraph::try_new(ressa_result) {
            Some(ms_graph) => ms_graph,
            None => return Err(Error::AppData("Could not create microservice graph".into())),
        };

        let microservices = ms_graph.nodes();
        let entities: Vec<_> = microservices
            .iter()
            .flat_map(|ms| &ms.ref_entities)
            .cloned()
            .collect();
        // let boundex_context_entities = get_bounded_context(&entities);
        // let entity_diagram = MermaidString::from(bounded_context_entities);

        let communication_diagram = MermaidString::from(ms_graph);

        //let microservices = microservices.into_iter().map(|ms| Microservice {
        //    name: ms.name,
        //    entity_diagram: Some(MermaidString::from(&ms.ref_entities)),
        //});

        //Ok(AppData {
        //    name: "???".into(),
        //    communication_diagram,
        //    entity_diagram: None,
        //    microservices,
        //})
        todo!()
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

        AppData::from_ressa_result(&result)
        // Clean up repos on disk on drop
    }
}
