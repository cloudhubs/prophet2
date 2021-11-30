//! Types for use across the prophet crates
use std::{collections::BTreeMap, str::FromStr};

use petgraph::graph::{DiGraph, NodeIndex};
use runestick::Value;
use source_code_parser::{ressa, ressa::RessaResult, Language};

/// A microservice detected from a ReSSA
#[derive(Debug)]
pub struct Microservice {
    pub name: String,
    pub language: Language,
    pub ref_entities: Vec<Entity>,
}

impl TryFrom<&BTreeMap<String, Value>> for Microservice {
    type Error = ressa::Error;

    /// Attempts to create a microservice from a ReSSA's object
    fn try_from(service: &BTreeMap<String, Value>) -> Result<Self, Self::Error> {
        let name = ressa::extract(service, "name", Value::into_string)?;
        let language =
            ressa::extract(service, "language", Value::into_string).map(Language::from)?;
        let ref_entities = ressa::extract_vec(service, "entities", Value::into_object)?
            .into_iter()
            .map(ressa::extract_object)
            .flat_map(|entity| Entity::try_from(&entity))
            .collect::<Vec<_>>();
        Ok(Microservice {
            name,
            language,
            ref_entities,
        })
    }
}

/// Represents a call between microservices
#[derive(Debug)]
pub enum MicroserviceCall {
    Http(http::Method),
    Rpc,
}

impl TryFrom<&BTreeMap<String, Value>> for MicroserviceCall {
    type Error = ressa::Error;

    /// Attempts to convert a ReSSA object to a microservice call
    fn try_from(call: &BTreeMap<String, Value>) -> Result<Self, Self::Error> {
        let ty = ressa::extract(call, "type", Value::into_string)?;
        let method = ressa::extract(call, "method", Value::into_string);
        let call = match method {
            Ok(method) if ty == "HTTP" => MicroserviceCall::Http(
                http::Method::from_str(&method)
                    .map_err(|_| ressa::Error::InvalidType("Bad HTTP method".into()))?,
            ),
            Err(_) if ty == "RPC" => MicroserviceCall::Rpc,
            _ => {
                return Err(ressa::Error::InvalidType(
                    "Bad microservice call type".into(),
                ))
            }
        };
        Ok(call)
    }
}

/// A graph of calls between microservices
#[derive(Debug)]
pub struct MicroserviceGraph(DiGraph<Microservice, MicroserviceCall>);

impl MicroserviceGraph {
    /// Attempts to create a microservice graph from a ReSSA result
    pub fn try_new(result: &RessaResult) -> Option<MicroserviceGraph> {
        let ctx = result.get("ctx")?;
        // Get the services shared vec from the context
        let services = ressa::extract_vec(ctx, "services", Value::into_object)
            .ok()?
            .into_iter()
            .map(ressa::extract_object)
            .collect::<Vec<_>>();

        // Create the graph with the service nodes
        let mut graph: DiGraph<Microservice, MicroserviceCall> = DiGraph::new();
        let indices = MicroserviceGraph::add_nodes(&mut graph, &services);

        // Get the calls each of the services makes
        let services = services.iter().flat_map(|service| {
            let name = Microservice::try_from(service)?.name;
            let calls = ressa::extract_vec(service, "calls", Value::into_object)?
                .into_iter()
                .map(ressa::result::extract_object)
                .collect::<Vec<_>>();
            Ok::<_, ressa::Error>((name, calls))
        });

        // Add directed edges between services in the graph
        for (service_name, calls) in services {
            let service_ndx = indices
                .iter()
                .find(|ndx| graph[**ndx].name == service_name)?;

            for call in calls.iter() {
                let called_name = ressa::extract(call, "name", Value::into_string).ok()?;
                let called_service_ndx = indices
                    .iter()
                    .find(|ndx| graph[**ndx].name == called_name)?;
                let call = call.try_into().ok()?;

                graph.add_edge(*service_ndx, *called_service_ndx, call);
            }
        }

        Some(MicroserviceGraph(graph))
    }

    fn add_nodes(
        graph: &mut DiGraph<Microservice, MicroserviceCall>,
        services: &[BTreeMap<String, Value>],
    ) -> Vec<NodeIndex> {
        services
            .iter()
            .flat_map(Microservice::try_from)
            .map(|node| graph.add_node(node))
            .collect::<Vec<_>>()
    }
}

/// Represents an entity from the ReSSA
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<Field>,
    pub ty: DatabaseType,
}

impl TryFrom<&BTreeMap<String, Value>> for Entity {
    type Error = ressa::Error;

    /// Attempts to create an Entity from a ReSSA object
    fn try_from(entity: &BTreeMap<String, Value>) -> Result<Self, Self::Error> {
        let name = ressa::extract(entity, "name", Value::into_string)?;
        let ty: DatabaseType = ressa::extract(entity, "type", Value::into_string)?.into();

        let fields = ressa::extract_vec(entity, "fields", Value::into_object)?
            .into_iter()
            .map(ressa::extract_object)
            .flat_map(|f| Field::try_from(&f))
            .collect::<Vec<_>>();

        Ok(Entity { name, fields, ty })
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum DatabaseType {
    MySQL,
    MongoDB,
    Unknown(String),
}

impl From<String> for DatabaseType {
    fn from(value: String) -> Self {
        match &*value {
            "MySQL" => DatabaseType::MySQL,
            "MongoDB" => DatabaseType::MongoDB,
            _ => DatabaseType::Unknown(value),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Field {
    pub name: String,
    pub ty: String,
}

impl TryFrom<&BTreeMap<String, Value>> for Field {
    type Error = ressa::Error;

    fn try_from(entity: &BTreeMap<String, Value>) -> Result<Self, Self::Error> {
        let name = ressa::extract(entity, "name", Value::into_string)?;
        let ty = ressa::extract(entity, "type", Value::into_string)?;
        Ok(Field { name, ty })
    }
}

#[derive(Debug)]
pub enum Multiplicity {
    // ...
}

#[derive(Debug)]
pub struct EntityGraph(DiGraph<Entity, Multiplicity>);

impl From<&[Entity]> for EntityGraph {
    fn from(_entities: &[Entity]) -> Self {
        todo!()
    }
}

impl AsRef<DiGraph<Entity, Multiplicity>> for EntityGraph {
    fn as_ref(&self) -> &DiGraph<Entity, Multiplicity> {
        &self.0
    }
}
