use std::ops::Add;

use derive_new::new;
use prophet_model::{Microservice, Entity, Field, DatabaseType};
use serde::{Deserialize, Serialize};

/// Request DTO:
#[derive(new, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundedContextRequest {
    context: BoundedContextSystem,
    use_wu_palmer: bool,
}

#[derive(new, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundedContextSystem {
    system_name: String,
    modules: Vec<BoundedContextModule>,
}

#[derive(new, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundedContextModule {
    name: String,
    entities: Vec<BoundedContextEntity>,
}

#[derive(new, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundedContextEntity {
    entity_name: String,
    fields: Vec<BoundedContextField>,
}

#[derive(new, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BoundedContextField {
    name: String,
    r#type: String,
}

impl From<Microservice> for BoundedContextSystem {
    fn from(ms: Microservice) -> Self {
        BoundedContextSystem {
            system_name: ms.name,
            modules: ms
                .ref_entities
                .into_iter()
                .map(|entity| BoundedContextModule::new(entity.name.clone(), vec![entity.into()]))
                .collect(),
        }
    }
}

impl From<Entity> for BoundedContextEntity {
    fn from(entity: Entity) -> Self {
        BoundedContextEntity {
            entity_name: entity.name,
            fields: entity.fields.into_iter().map(|field| field.into()).collect(),
        }
    }
}

impl From<Field> for BoundedContextField {
    fn from(field: Field) -> Self {
        BoundedContextField {
            name: field.name,
            r#type: field.ty,
        }
    }
}

/// Response DTO:
#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub(crate) struct MergedEntitySystem {
    system_name: String,
    bounded_context_entities: Vec<MergedEntity>,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub(crate) struct MergedEntity {
    entity_name: MergedName,
    fields: Vec<MergedField>,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub(crate) struct MergedName {
    name: String,
    full_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub(crate) struct MergedField {
    name: MergedName,
    r#type: String,
    reference: bool,
    collection: bool,
}

impl From<MergedEntitySystem> for Vec<Entity> {
    fn from(mes: MergedEntitySystem) -> Self {
        mes.bounded_context_entities.into_iter().map(|entity| entity.into()).collect()
    }
}
impl From<MergedEntity> for Entity {
    fn from(me: MergedEntity) -> Self {
        Entity {
            name: me.entity_name.full_name,
            fields: me.fields.into_iter().map(|field| field.into()).collect(),
            ty: DatabaseType::Unknown(String::new()),
        }
    }
}
impl From<MergedField> for Field {
    fn from(mf: MergedField) -> Self {
        Field {
            name: mf.name.full_name,
            ty: mf.r#type,
            is_collection: mf.collection,
        }
    }
}
