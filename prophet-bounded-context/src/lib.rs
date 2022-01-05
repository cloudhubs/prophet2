use actix_web::{
    client::{self, Client},
    http::StatusCode,
};
use once_cell::sync::OnceCell;
use prophet_model::{Entity, EntityGraph, Microservice};

use compat::*;
use structopt::StructOpt;
pub(crate) mod compat;

/// CL arguments to set location of bounded-context service
#[derive(StructOpt)]
struct Opt {
    #[structopt(long, short, default_value = "127.0.0.1")]
    host: String,
    #[structopt(long, short, default_value = "8080")]
    port: i32,
}
static OPTS: OnceCell<Opt> = OnceCell::new();

/// Retrieve CL arguments identifying bounded-context service
fn get_opts() -> &'static Opt {
    OPTS.get_or_init(|| Opt::from_args())
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("ReSSA Error: {0}")]
    Ressa(String),
    #[error("Remote Call Error: {0}")]
    RemoteCall(String),
    #[error("Deserialization Error: {0}")]
    Deserialize(String),
    #[error("Conversion to Graph Failed")]
    Conversion,
}

/// Convert the ReSSA's output into a bounded context, using an external service
pub async fn get_bounded_context(ms: Microservice) -> Result<EntityGraph, Error> {
    let req = BoundedContextRequest::new(ms.into(), false);
    let entities: Vec<Entity> = retrieve(req).await?.into();
    match EntityGraph::try_new(&entities) {
        Some(graph) => Ok(graph),
        None => Err(Error::Conversion),
    }
}

/// Make the API call to merge entities
async fn retrieve(req: BoundedContextRequest) -> Result<MergedEntitySystem, Error> {
    let client = Client::default();
    let options = get_opts();

    // Make request and handle error (if occurred)
    let result = client
        .post(format!("http://{}:{}/", options.host, options.port))
        .header("User-Agent", "actix-web/3.0")
        .send_json(&req)
        .await
        .map_err(|err| Error::RemoteCall(err.to_string()))?;

    // Handle error response status
    let mut result = match result.status() {
        StatusCode::OK => Ok(result),
        err => Err(Error::RemoteCall(err.to_string())),
    }?;

    // Handle errors from extracting body
    let body = match result.body().await {
        Ok(result) => Ok(result),
        Err(err) => Err(Error::RemoteCall(err.to_string())),
    }?;

    // Decode and return
    let body = serde_json::from_slice::<'_, MergedEntitySystem>(&body)
        .map_err(|err| Error::Deserialize(err.to_string()))?;
    Ok(body)
}
