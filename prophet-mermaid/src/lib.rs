use prophet_model::{
    Cardinality, Edge, Edges, Entity, EntityGraph, Microservice, MicroserviceCall,
    MicroserviceGraph,
};
use serde::Serialize;
use std::fmt::Write;

/// A mermaid string to represent a microservice communication diagram
/// or an entity diagram
#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct MermaidString(String);

impl MermaidString {
    fn from_graph<N, E, F1, F2>(
        edges: Edges<N, E>,
        header: &str,
        mut write_node: F1,
        mut write_edge: F2,
    ) -> Self
    where
        F1: FnMut(&mut String, &N) -> std::fmt::Result,
        F2: FnMut(&mut String, &Edge<N, E>) -> std::fmt::Result,
    {
        let edges: Vec<_> = edges.into_inner();
        let mut mermaid = format!("{}\n", header);

        for edge in edges {
            write_node(&mut mermaid, &edge.from).unwrap();
            write_edge(&mut mermaid, &edge).unwrap();
        }

        Self(mermaid)
    }
}

/*
graph TD
SourceMicroservice -->|"HTTP Verb: GET<br/>Arguments: ...<br/>Endpoint function ..."| TargetMicroservice
...
 */

impl From<MicroserviceGraph> for MermaidString {
    fn from(graph: MicroserviceGraph) -> Self {
        fn write_ms_string(_w: &mut impl Write, _ms: &Microservice) -> std::fmt::Result {
            Ok(())
        }

        fn write_ms_edge(
            w: &mut impl Write,
            edge: &Edge<Microservice, MicroserviceCall>,
        ) -> std::fmt::Result {
            Ok(())
        }

        MermaidString::from_graph(graph.edges(), "graph TD", write_ms_string, write_ms_edge)
    }
}

impl From<EntityGraph> for MermaidString {
    fn from(graph: EntityGraph) -> Self {
        fn write_entity_string(w: &mut impl Write, entity: &Entity) -> std::fmt::Result {
            writeln!(w, "class {} {{", entity.name)?;
            for field in entity.fields.iter() {
                let ty = if field.is_collection {
                    format!("List<{}>", field.ty)
                } else {
                    field.ty.clone()
                };
                writeln!(w, "+{} {}", ty, field.name)?;
            }
            writeln!(w, "}}")?;
            Ok(())
        }

        fn write_entity_edge(
            w: &mut impl Write,
            edge: &Edge<Entity, Cardinality>,
        ) -> std::fmt::Result {
            let cardinality = edge.weight.to_string();
            writeln!(
                w,
                r#"{} "1" --> "{}" {}"#,
                edge.from.name, cardinality, edge.to.name
            )?;
            Ok(())
        }

        MermaidString::from_graph(
            graph.edges(),
            "classDiagram",
            write_entity_string,
            write_entity_edge,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prophet_model::*;
    use test_case::test_case;

    const ENTITY_MERMAID: &str = r#"classDiagram
class EntityOne {
+List<EntityTwo> f1
}
EntityOne "1" --> "*" EntityTwo
class EntityTwo {
+int x
+EntityOne other
}
EntityTwo "1" --> "1" EntityOne
"#;

    fn get_entity_graph() -> EntityGraph {
        EntityGraph::try_new(&[
            Entity::new(
                "EntityOne",
                vec![Field::new("f1", "EntityTwo", true)],
                DatabaseType::MySQL,
            ),
            Entity::new(
                "EntityTwo",
                vec![
                    Field::new("x", "int", false),
                    Field::new("other", "EntityOne", false),
                ],
                DatabaseType::MySQL,
            ),
        ])
        .unwrap()
    }

    #[test_case(get_entity_graph() => MermaidString(ENTITY_MERMAID.to_string()) ; "one_to_many")]
    fn from_entity_graph_test(graph: impl Into<MermaidString>) -> MermaidString {
        graph.into()
    }
}
