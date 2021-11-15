use serde::Serialize;

/// A mermaid string to represent a microservice communication diagram
/// or an entity diagram
#[derive(Debug, Default, Serialize)]
pub struct MermaidString(String);
