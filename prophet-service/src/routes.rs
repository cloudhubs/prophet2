use actix_web::{error, post, web, Error, HttpResponse};
use prophet::{AppData, Repositories};

#[post("/analyze")]
pub async fn analyze(payload: web::Json<Repositories>) -> Result<HttpResponse, Error> {
    let app_data = AppData::from_repositories(payload.into_inner())
        .map_err(error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(app_data))
}
