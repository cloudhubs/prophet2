use prophet_model::{DatabaseType, Entity, EntityGraph, Field, Microservice};
use source_code_parser::Language;

#[actix_web::main]
async fn main() {
    let entity = Entity {
        name: "Entity1".to_string(),
        fields: vec![Field {
            name: "FieldA".to_string(),
            ty: "Foo".to_string(),
            is_collection: false,
        }],
        ty: DatabaseType::MongoDB,
    };
    let oracle = match EntityGraph::try_new(&vec![ Entity {
        ty: DatabaseType::Unknown("".to_string()),
        ..entity.clone()
    }]) {
        Some(graph) => graph,
        None => panic!("Cannot convert oracle entities"),
    };
    let ms = Microservice {
        name: "dummy".to_string(),
        language: Language::Java,
        ref_entities: vec![entity.clone(), entity],
    };

    let result = prophet_bounded_context::get_bounded_context(ms).await.unwrap();
    println!("Expected: {:#?}", oracle);
    println!("Result: {:#?}", result);
}
