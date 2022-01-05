//! Types for use across the prophet crates
use std::{collections::BTreeMap, str::FromStr};

use petgraph::{
    graph::{DiGraph, NodeIndex},
    visit::EdgeRef,
};
use runestick::Value;
use source_code_parser::{ressa, ressa::RessaResult, Language};
use strum::Display;

/// A microservice detected from a ReSSA
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, Display)]
pub enum MicroserviceCall {
    Http(http::Method),
    #[strum(serialize = "RPC")]
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
        let indices = add_nodes(&mut graph, &services);

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

    /// Gets the directed edges for the microservice graph
    pub fn edges(&self) -> Edges<Microservice, MicroserviceCall> {
        Edges::from(&self.0)
    }

    // Gets all of the nodes in the graph
    pub fn nodes(&self) -> Vec<Microservice> {
        get_nodes(&self.0)
    }
}

fn get_nodes<N: Clone, E>(graph: &DiGraph<N, E>) -> Vec<N> {
    graph.node_indices().map(|ndx| graph[ndx].clone()).collect()
}

fn add_nodes<'a, N, E>(
    graph: &mut DiGraph<N, E>,
    services: &'a [BTreeMap<String, Value>],
) -> Vec<NodeIndex>
where
    N: TryFrom<&'a BTreeMap<String, Value>>,
{
    add_nodes_inner(graph, services.iter().flat_map(N::try_from))
}

fn add_nodes_inner<N, E>(
    graph: &mut DiGraph<N, E>,
    services: impl Iterator<Item = N>,
) -> Vec<NodeIndex> {
    services
        .map(|node| graph.add_node(node))
        .collect::<Vec<_>>()
}

impl AsRef<DiGraph<Microservice, MicroserviceCall>> for MicroserviceGraph {
    fn as_ref(&self) -> &DiGraph<Microservice, MicroserviceCall> {
        &self.0
    }
}

/// Represents an entity from the ReSSA
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<Field>,
    pub ty: DatabaseType,
}

impl Entity {
    pub fn new(name: impl ToString, fields: Vec<Field>, ty: DatabaseType) -> Self {
        Entity {
            name: name.to_string(),
            fields,
            ty,
        }
    }
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

#[derive(Debug, PartialEq, Eq, Hash, Clone, Display)]
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

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Field {
    pub name: String,
    pub ty: String,
    pub is_collection: bool,
}

impl Field {
    pub fn new(name: impl ToString, ty: impl ToString, is_collection: bool) -> Self {
        Field {
            name: name.to_string(),
            ty: ty.to_string(),
            is_collection,
        }
    }
}

impl TryFrom<&BTreeMap<String, Value>> for Field {
    type Error = ressa::Error;

    fn try_from(entity: &BTreeMap<String, Value>) -> Result<Self, Self::Error> {
        let name = ressa::extract(entity, "name", Value::into_string)?;
        let ty = ressa::extract(entity, "type", Value::into_string)?;
        let is_collection = ressa::extract_primitive(entity, "is_collection", Value::into_bool)?;
        Ok(Field {
            name,
            ty,
            is_collection,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Cardinality {
    One,
    Many,
}

impl ToString for Cardinality {
    fn to_string(&self) -> String {
        use Cardinality::*;
        match self {
            One => "1",
            Many => "*",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct EntityGraph(DiGraph<Entity, Cardinality>);

impl EntityGraph {
    /// Attempts to create an entity graph from a list of combined Entities
    pub fn try_new(entities: &[Entity]) -> Option<EntityGraph> {
        let mut graph = DiGraph::new();
        let indices = add_nodes_inner(&mut graph, entities.iter().cloned());

        // Add entity nodes to the graph
        for entity in entities {
            let entity_ndx = indices
                .iter()
                .find(|ndx| graph[**ndx].name == entity.name)?;

            for field in entity.fields.iter() {
                // Get the matching entity for the field
                let other_entity_ndx = indices.iter().find(|ndx| graph[**ndx].name == field.ty);
                let other_entity_ndx = match other_entity_ndx {
                    Some(ndx) => ndx,
                    _ => continue,
                };

                let other_cardinality = if field.is_collection {
                    Cardinality::Many
                } else {
                    Cardinality::One
                };

                graph.add_edge(*entity_ndx, *other_entity_ndx, other_cardinality);
                //graph.add_edge(*other_entity_ndx, *entity_ndx, Cardinality::One);
            }
        }

        Some(EntityGraph(graph))
    }

    /// Gets the directed edges for the entity graph
    pub fn edges(&self) -> Edges<Entity, Cardinality> {
        Edges::from(&self.0)
    }

    /// Gets all of the nodes in the graph
    pub fn nodes(&self) -> Vec<Entity> {
        get_nodes(&self.0)
    }

    /// Filters an entity graph to contain certain entities
    pub fn filter_entities(&mut self, entities: &[Entity]) {
        let graph = &mut self.0;

        // Graph::remove_node invalidates the last node index, so we need to repeatedly find the
        // entities that should be filtered out so we have valid indices that can remove the nodes.
        while let Some(ndx) = graph.node_indices().find_map(|ndx| {
            if entities.iter().any(|e| *e == graph[ndx]) {
                Some(ndx)
            } else {
                None
            }
        }) {
            // We know the node is in the list since we just found its index and the graph has not
            // been mutated elsewhere before this statement, so the index is valid
            graph.remove_node(ndx);
        }
    }
}

impl AsRef<DiGraph<Entity, Cardinality>> for EntityGraph {
    fn as_ref(&self) -> &DiGraph<Entity, Cardinality> {
        &self.0
    }
}

/// The directed edges in a graph
#[derive(Debug)]
pub struct Edges<N, E>(Vec<Edge<N, E>>);

impl<N, E> Edges<N, E> {
    /// Converts the edges into its inner representation
    pub fn into_inner(self) -> Vec<Edge<N, E>> {
        self.0
    }
}

/// A directed edge
#[derive(Debug)]
pub struct Edge<N, E> {
    pub from: N,
    pub to: N,
    pub weight: E,
}

impl<N, E> From<&DiGraph<N, E>> for Edges<N, E>
where
    N: Clone,
    E: Clone + std::fmt::Debug,
{
    fn from(graph: &DiGraph<N, E>) -> Self {
        // Get all directed edges in the graph and map them to our Edges structure
        Edges(
            graph
                .edge_references()
                .map(|edge_ref| {
                    let weight = edge_ref.weight().clone();
                    let from = graph[edge_ref.source()].clone();
                    let to = graph[edge_ref.target()].clone();
                    Edge { from, to, weight }
                })
                .collect::<Vec<_>>(),
        )
    }
}
