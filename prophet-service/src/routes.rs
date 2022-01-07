use actix_web::{error, post, web, Error, HttpResponse};
use prophet::{AppData, Repositories};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AnalysisBody {
    ressa_dir: String,
    repositories: Repositories,
}

#[post("/analyze")]
pub async fn analyze(payload: web::Json<AnalysisBody>) -> Result<HttpResponse, Error> {
    let payload = payload.into_inner();
    let app_data = AppData::from_repositories(payload.repositories, payload.ressa_dir)
        .await
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(app_data))
}
