use prophet_model::{DatabaseType, Entity, EntityGraph, Field, Microservice};
use source_code_parser::Language;

/// TODO replace with a proper integration test
#[actix_web::main]
async fn main() {
    let entity_a = Entity {
        name: "Entity1".to_string(),
        fields: vec![Field {
            name: "FieldA".to_string(),
            ty: "Foo".to_string(),
            is_collection: false,
        }],
        ty: DatabaseType::MongoDB,
    };
    let entity_b = Entity {
        name: "AnotherEntity".to_string(),
        fields: vec![Field {
            name: "AnotherField".to_string(),
            ty: "Waa".to_string(),
            is_collection: true,
        }],
        ty: DatabaseType::MySQL,
    };

    let oracle = match EntityGraph::try_new(&vec![
        Entity {
            ty: DatabaseType::Unknown("".to_string()),
            ..entity_a.clone()
        },
        Entity {
            ty: DatabaseType::Unknown("".to_string()),
            ..entity_b.clone()
        },
    ]) {
        Some(graph) => graph,
        None => panic!("Cannot convert oracle entities"),
    };
    let ms = Microservice {
        name: "dummy".to_string(),
        language: Language::Java,
        ref_entities: vec![entity_a.clone(), entity_a, entity_b],
    };

    let result = prophet_bounded_context::get_bounded_context(ms)
        .await
        .unwrap();
    println!("Expected: {:#?}", oracle);
    println!("Result: {:#?}", result);
}
