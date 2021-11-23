use std::cell::Ref;

use petgraph::graph::DiGraph;
use source_code_parser::{
    coerce, extract, extract_object, extract_vec, ressa::RessaResult, Language,
};

#[derive(Default, Debug)]
pub struct Microservice {
    pub name: String,
    pub language: Language,
    pub ref_entities: Vec<Entity>,
}

#[derive(Debug)]
pub enum MicroserviceCall {
    Http(HttpMethod),
    Rpc,
}

// TODO use a crate for this...
#[derive(Debug)]
pub enum HttpMethod {
    Post,
    Get, //...
}

#[derive(Debug)]
pub struct MicroserviceGraph(DiGraph<Microservice, MicroserviceCall>);

impl MicroserviceGraph {
    pub fn from_ressa_result(result: &RessaResult) -> Option<MicroserviceGraph> {
        let ctx = result.get("ctx")?;
        // Get the services shared vec from the context
        // let services = extract_vec!(ctx, "services", into_object).ok()?;

        let services = ctx.get("services")?.into_vec().ok()?;
        // Get a reference to the inner services vec
        let services = services.into_ref().ok()?;

        let nodes = services
            .iter()
            // .flat_map(|service| )
            .flat_map(|service| service.into_object())
            .flat_map(|service| service.into_ref())
            .map(|service| {
                let name = extract_object(service);
            })
            // .flat_map(|service| {
            //     let obj = service.into_inner();
            // })
            // .map(|service| {
            //     service.
            // })
            .map(|(name, language)| Microservice { name, language })
            .collect::<Vec<Microservice>>();

        let mut graph = DiGraph::new();
        // ...

        Some(MicroserviceGraph(graph))
    }

    fn into_service_language_refs(service: Ref<runestick::Object>) -> Option<(String, Language)> {
        let name = service.get("name")?.into_string().ok()?.into_ref().ok()?;
        let language = service
            .get("language")?
            .into_string()
            .ok()?
            .into_ref()
            .ok()?;
        Some((name.clone(), language.clone().into()))
    }
}

#[derive(Debug)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<Field>,
    pub ty: DatabaseType,
}

#[derive(Debug)]
pub enum DatabaseType {
    MySQL,
    MongoDB,
}

#[derive(Debug)]
pub struct Field {
    pub name: String,
    pub ty: String,
}

#[derive(Debug)]
pub enum Multiplicity {
    // ...
}

#[derive(Debug)]
pub struct EntityGraph(DiGraph<Entity, Multiplicity>);

impl EntityGraph {
    pub fn from_ressa_result(result: &RessaResult) -> Option<EntityGraph> {
        todo!()
    }
}
