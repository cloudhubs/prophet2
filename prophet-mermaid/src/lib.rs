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
    fn from_graph<N, E, FNode, FEdge>(
        nodes: Vec<N>,
        edges: Edges<N, E>,
        header: &str,
        write_node: Option<FNode>,
        mut write_edge: FEdge,
        write_orphan: Option<FNode>,
    ) -> Self
    where
        N: PartialEq,
        FNode: FnMut(&mut String, &N) -> std::fmt::Result,
        FEdge: FnMut(&mut String, &Edge<N, E>) -> std::fmt::Result,
    {
        let edges: Vec<_> = edges.into_inner();
        let mut mermaid = format!("{}\n", header);

        if let Some(mut write_node) = write_node {
            // Write the node on its own
            for node in nodes.iter() {
                write_node(&mut mermaid, node).unwrap();
            }
        }

        for edge in edges.iter() {
            // Write any edges in the graph
            write_edge(&mut mermaid, edge).unwrap();
        }

        if let Some(mut write_orphan) = write_orphan {
            // Write any nodes that had no edges
            let orphans = nodes.iter().filter(|node| {
                !edges
                    .iter()
                    .any(|edge| edge.from == **node || edge.to == **node)
            });
            for orphan in orphans {
                write_orphan(&mut mermaid, orphan).unwrap();
            }
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
        fn write_edge(
            w: &mut impl Write,
            from: &str,
            to: &str,
            label: Option<&str>,
        ) -> std::fmt::Result {
            // Write the call edge with any extra information if available
            match label {
                Some(label) => writeln!(w, "{} -->|\"{}\"| {}", to, label, from),
                None => writeln!(w, "{} --> {}", to, from),
            }
        }

        fn write_ms_edge(
            w: &mut impl Write,
            edge: &Edge<Microservice, MicroserviceCall>,
        ) -> std::fmt::Result {
            // Note: it looks like the only information we have defined for the calls at the moment
            // is just the HTTP method or RPC call indicator. As seen in the comment above, there
            // was previously extra information like arguments and specific endpoints.
            let label = match &edge.weight {
                call_ty @ MicroserviceCall::Http(_) => {
                    format!("HTTP Verb: {}", call_ty)
                }
                call_ty @ MicroserviceCall::Rpc => format!("{}", call_ty),
            };
            write_edge(
                w,
                edge.from.name.as_str(),
                edge.to.name.as_str(),
                Some(label.as_str()),
            )
        }

        fn write_ms_orphan(w: &mut impl Write, node: &Microservice) -> std::fmt::Result {
            write_edge(w, node.name.as_str(), "N/A", None)
        }

        MermaidString::from_graph(
            graph.nodes(),
            graph.edges(),
            "graph TD",
            None,
            write_ms_edge,
            Some(write_ms_orphan),
        )
    }
}

/*
classDiagram
class A {
<<db_type_annotation>>
+Type name
...
}
A "1" --> "*" B
...
 */
impl From<EntityGraph> for MermaidString {
    fn from(graph: EntityGraph) -> Self {
        fn write_entity_string(w: &mut impl Write, entity: &Entity) -> std::fmt::Result {
            writeln!(w, "class {} {{", entity.name)?;
            // Write the database type
            writeln!(w, "<<{}>>", entity.ty)?;
            // Write the fields
            for field in entity.fields.iter() {
                let ty = if field.is_collection {
                    format!("List<{}>", field.ty)
                } else {
                    field.ty.clone()
                };
                writeln!(w, "+{} {}", ty, field.name)?;
            }
            writeln!(w, "}}")
        }

        fn write_entity_edge(
            w: &mut impl Write,
            edge: &Edge<Entity, Cardinality>,
        ) -> std::fmt::Result {
            // Write the relation represented by the edge
            let cardinality = edge.weight.to_string();
            writeln!(
                w,
                r#"{} "1" --> "{}" {}"#,
                edge.from.name, cardinality, edge.to.name
            )
        }

        MermaidString::from_graph(
            graph.nodes(),
            graph.edges(),
            "classDiagram",
            Some(write_entity_string),
            write_entity_edge,
            None,
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
<<MySQL>>
+List<EntityTwo> f1
}
class EntityTwo {
<<MySQL>>
+int x
+EntityOne other
}
EntityOne "1" --> "*" EntityTwo
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
