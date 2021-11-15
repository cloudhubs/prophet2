use petgraph::graph::DiGraph;
use source_code_parser::ressa::RessaResult;

// We may want to move this into a separate crate to avoid dependency cycle issues

#[derive(Default, Debug)]
pub struct Microservice;

#[derive(Default, Debug)]
pub struct MicroserviceGraph(DiGraph<Microservice, Microservice>);

impl From<RessaResult> for MicroserviceGraph {
    fn from(_: RessaResult) -> Self {
        todo!()
    }
}

#[derive(Default, Debug)]
pub struct Entity;

#[derive(Default, Debug)]
pub struct EntityGraph(DiGraph<Entity, Entity>);

impl From<RessaResult> for EntityGraph {
    fn from(_: RessaResult) -> Self {
        todo!()
    }
}
