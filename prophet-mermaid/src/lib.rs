use serde::Serialize;

/// A mermaid string to represent a microservice communication diagram
/// or an entity diagram
#[derive(Debug, Default, Serialize)]
pub struct MermaidString(String);

impl MermaidString {
    // TODO determine methods to create the mermaid string
}
