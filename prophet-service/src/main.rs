use actix_web::{middleware::Logger, web, App, FromRequest, HttpServer};
use prophet::Repositories;
use structopt::StructOpt;

mod routes;
use routes::*;

#[derive(StructOpt)]
struct Opt {
    #[structopt(long, short, default_value = "127.0.0.1")]
    host: String,
    #[structopt(long, short, default_value = "8080")]
    port: i32,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let opt = Opt::from_args();
    let addr = format!("{}:{}", opt.host, opt.port);

    HttpServer::new(|| {
        App::new()
            .service(analyze)
            .wrap(Logger::default())
            .app_data(web::Json::<Repositories>::configure(|cfg| {
                cfg.limit(1024 * 1024 * 4)
            }))
    })
    .bind(addr)?
    .run()
    .await
}
