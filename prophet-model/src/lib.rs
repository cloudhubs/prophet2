use std::{collections::HashSet, str::FromStr};

use petgraph::graph::DiGraph;
use source_code_parser::{ressa, ressa::RessaResult, Language};

#[derive(Debug)]
pub struct Microservice<'e> {
    pub name: String,
    pub language: Language,
    pub ref_entities: Vec<&'e Entity>,
}

#[derive(Debug)]
pub enum MicroserviceCall {
    Http(http::Method),
    Rpc,
}

#[derive(Debug)]
pub struct MicroserviceGraph<'e>(DiGraph<Microservice<'e>, MicroserviceCall>);

impl<'e> MicroserviceGraph<'e> {
    pub fn try_new(
        result: &RessaResult,
        entities: &'e EntityGraph,
    ) -> Option<MicroserviceGraph<'e>> {
        let ctx = result.get("ctx")?;
        // Get the services shared vec from the context
        let services = ressa::extract_vec(ctx, "services", |v| v.into_object())
            .ok()?
            .into_iter()
            .map(ressa::extract_object)
            .collect::<Vec<_>>();

        let entities = entities.as_ref().node_weights().collect::<Vec<_>>();

        let mut nodes = services
            .iter()
            .flat_map(|service| {
                let name = ressa::extract(service, "name", |v| v.into_string())?;
                let lang =
                    ressa::extract(service, "language", |v| v.into_string()).map(Language::from)?;
                let entity_names = ressa::extract_vec(service, "entities", |v| v.into_string())?
                    .into_iter()
                    .collect::<HashSet<_>>();

                let entities = entities
                    .iter()
                    .filter(|entity| entity_names.get(&entity.name).is_some())
                    .cloned()
                    .collect::<Vec<_>>();

                Ok::<_, ressa::Error>((name, lang, entities))
            })
            .map(|(name, language, ref_entities)| Microservice {
                name,
                language,
                ref_entities,
            })
            .collect::<Vec<_>>();

        let mut graph: DiGraph<Microservice, MicroserviceCall> = DiGraph::new();
        let indices = nodes
            .into_iter()
            .map(|node| graph.add_node(node))
            .collect::<Vec<_>>();

        let services = services.iter().flat_map(|service| {
            let name = ressa::extract(service, "name", |v| v.into_string())?;
            let calls = ressa::extract_vec(service, "calls", |v| v.into_object())?
                .into_iter()
                .map(ressa::result::extract_object)
                .collect::<Vec<_>>();
            Ok::<_, ressa::Error>((name, calls))
        });
        for (service_name, calls) in services {
            // let service = nodes
            //     .iter_mut()
            //     .find(|service| service.name == service_name)?;

            // for call in calls.iter() {
            //     let called_name = ressa::extract(call, "name", |v| v.into_string()).ok()?;
            //     let called_service = nodes
            //         .iter_mut()
            //         .find(|service| service.name == called_name)?;

            //     let ty = ressa::extract(call, "type", |v| v.into_string()).ok()?;
            //     let method = ressa::extract(call, "method", |v| v.into_string()).ok();
            //     let call = match method {
            //         Some(method) => MicroserviceCall::Http(http::Method::from_str(&method).ok()?),
            //         None => MicroserviceCall::Rpc,
            //     };
            // }
        }

        // ...

        Some(MicroserviceGraph(graph))
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

impl AsRef<DiGraph<Entity, Multiplicity>> for EntityGraph {
    fn as_ref(&self) -> &DiGraph<Entity, Multiplicity> {
        &self.0
    }
}
