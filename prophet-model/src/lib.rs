use petgraph::graph::DiGraph;
use source_code_parser::{ressa, ressa::RessaResult, Language};

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
        let services = ressa::extract_vec(ctx, "services", |v| v.into_object())
            .ok()?
            .into_iter()
            .map(ressa::extract_object)
            .collect::<Vec<_>>();

        let nodes = services
            .iter()
            .flat_map(|service| {
                let name = ressa::extract(service, "name", |v| v.into_string());
                let lang =
                    ressa::extract(service, "language", |v| v.into_string()).map(Language::from);
                match (name, lang) {
                    (Ok(name), Ok(lang)) => Ok((name, lang)),
                    (Err(err), _) | (_, Err(err)) => Err(err),
                }
            })
            .map(|(name, language)| Microservice {
                name,
                language,
                // TODO
                ref_entities: vec![],
            })
            .collect::<Vec<_>>();

        let mut graph = DiGraph::new();
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
