use std::{
    collections::{BTreeMap, HashSet},
    str::FromStr,
};

use petgraph::graph::{DiGraph, NodeIndex};
use runestick::Value;
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

impl TryFrom<&BTreeMap<String, Value>> for MicroserviceCall {
    type Error = ressa::Error;

    fn try_from(call: &BTreeMap<String, Value>) -> Result<Self, Self::Error> {
        // let ty = ressa::extract(call, "type", |v| v.into_string())?;
        let method = ressa::extract(call, "method", |v| v.into_string());
        let call = match method {
            Ok(method) => MicroserviceCall::Http(
                http::Method::from_str(&method)
                    .map_err(|_| ressa::Error::InvalidType("Bad HTTP method".into()))?,
            ),
            Err(_) => MicroserviceCall::Rpc,
        };
        Ok(call)
    }
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

        let mut graph: DiGraph<Microservice, MicroserviceCall> = DiGraph::new();
        let indices = MicroserviceGraph::add_nodes(&mut graph, &services, &entities);

        let services = services.iter().flat_map(|service| {
            let name = ressa::extract(service, "name", |v| v.into_string())?;
            let calls = ressa::extract_vec(service, "calls", |v| v.into_object())?
                .into_iter()
                .map(ressa::result::extract_object)
                .collect::<Vec<_>>();
            Ok::<_, ressa::Error>((name, calls))
        });
        for (service_name, calls) in services {
            let service_ndx = indices
                .iter()
                .find(|ndx| graph[**ndx].name == service_name)?;

            for call in calls.iter() {
                let called_name = ressa::extract(call, "name", |v| v.into_string()).ok()?;
                let called_service_ndx = indices
                    .iter()
                    .find(|ndx| graph[**ndx].name == called_name)?;
                let call = call.try_into().ok()?;

                graph.add_edge(*service_ndx, *called_service_ndx, call);
            }
        }

        // ...

        Some(MicroserviceGraph(graph))
    }

    fn add_nodes(
        graph: &mut DiGraph<Microservice<'e>, MicroserviceCall>,
        services: &[BTreeMap<String, Value>],
        entities: &[&'e Entity],
    ) -> Vec<NodeIndex> {
        services
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
            .map(|node| graph.add_node(node))
            .collect::<Vec<_>>()
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
