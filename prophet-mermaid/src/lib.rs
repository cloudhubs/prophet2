use prophet_model::{Cardinality, Edge, Entity, EntityGraph, MicroserviceGraph};
use serde::Serialize;
use std::fmt::Write;

/// A mermaid string to represent a microservice communication diagram
/// or an entity diagram
#[derive(Debug, Default, PartialEq, Eq, Serialize)]
pub struct MermaidString(String);

impl From<MicroserviceGraph> for MermaidString {
    fn from(graph: MicroserviceGraph) -> Self {
        let _edges = graph.edges().into_inner();
        let mermaid = "graphTD\n".to_string();

        // TODO

        Self(mermaid)
    }
}

impl From<EntityGraph> for MermaidString {
    fn from(graph: EntityGraph) -> Self {
        let edges = graph.edges().into_inner();
        let mut mermaid = "classDiagram\n".to_string();

        for edge in edges {
            write_entity_string(&mut mermaid, &edge.from).unwrap();
            write_entity_edge(&mut mermaid, &edge).unwrap();
        }

        Self(mermaid)
    }
}

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

fn write_entity_edge(w: &mut impl Write, edge: &Edge<Entity, Cardinality>) -> std::fmt::Result {
    let cardinality = edge.weight.to_string();
    writeln!(
        w,
        r#"{} "1" --> "{}" {}"#,
        edge.from.name, cardinality, edge.to.name
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use prophet_model::DatabaseType::MySQL;
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
Entity
EntityTwo "1" --> "1" EntityOne
"#;

    fn get_entity_graph() -> EntityGraph {
        EntityGraph::try_new(&[
            Entity::new(
                "EntityOne",
                vec![Field::new("f1", "EntityTwo", true)],
                MySQL,
            ),
            Entity::new(
                "EntityTwo",
                vec![
                    Field::new("x", "int", false),
                    Field::new("other", "EntityOne", false),
                ],
                MySQL,
            ),
        ])
        .unwrap()
    }

    #[test_case(get_entity_graph() => MermaidString(ENTITY_MERMAID.to_string()) ; "one_to_many")]
    fn from_entity_graph_test(graph: impl Into<MermaidString>) -> MermaidString {
        graph.into()
    }
}
