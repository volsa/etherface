mod v1;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpServer;
use etherface_lib::database::handler::DatabaseClientPooled;
use openssl::ssl::SslAcceptor;
use openssl::ssl::SslFiletype;
use openssl::ssl::SslMethod;
use v1::AppState;

const PATH_PRIVATE_KEY: &str = "/etc/letsencrypt/live/api.etherface.io/privkey.pem";
const PATH_CERTIFICATE: &str = "/etc/letsencrypt/live/api.etherface.io/fullchain.pem";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file(PATH_PRIVATE_KEY, SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file(PATH_CERTIFICATE).unwrap();

    let state = web::Data::new(AppState {
        dbc: DatabaseClientPooled::new().unwrap(),
    });

    HttpServer::new(move || {
        App::new().app_data(state.clone()).service(
            web::scope("/v1")
                .service(v1::signatures_by_text)
                .service(v1::signatures_by_hash)
                .service(v1::sources_github)
                .service(v1::sources_etherscan)
                .service(v1::statistics)
                .wrap(Cors::permissive())
                .wrap(Logger::new("(%Ts) %a: %r").log_target("v1::logger")),
        )
    })
    .bind_openssl("65.21.54.11:443", builder)?
    .run()
    .await
}
