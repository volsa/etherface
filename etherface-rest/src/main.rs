use actix_cors::Cors;
use actix_web::get;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use etherface_lib::database::handler::DatabaseClientPooled;
use etherface_lib::model::views::ViewSignatureCountStatistics;
use etherface_lib::model::views::ViewSignatureInsertRate;
use etherface_lib::model::views::ViewSignatureKindDistribution;
use etherface_lib::model::views::ViewSignaturesPopularOnGithub;
use etherface_lib::model::SignatureKind;
use openssl::ssl::SslAcceptor;
use openssl::ssl::SslFiletype;
use openssl::ssl::SslMethod;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    All,
    Function,
    Event,
    Error,
}

#[derive(Deserialize)]
struct ContentPath {
    input: String,
    kind: Kind,
    page: i64,
}

#[derive(Deserialize)]
struct SourcePath {
    signature_id: i32,
    kind: Kind,
    page: i64,
}

struct AppState {
    dbc: DatabaseClientPooled,
}

#[get("/v1/signatures/text/{kind}/{input}/{page}")]
async fn signatures_by_text(path: web::Path<ContentPath>, state: web::Data<AppState>) -> impl Responder {
    if path.input.trim().len() < 3 {
        return HttpResponse::BadRequest().body("Query must have at least 3 characters");
    }

    let kind = match path.kind {
        Kind::All => None,
        Kind::Function => Some(SignatureKind::Function),
        Kind::Event => Some(SignatureKind::Event),
        Kind::Error => Some(SignatureKind::Error),
    };

    match state.dbc.rest().signatures_where_text_starts_with(&path.input, kind, path.page) {
        Some(signatures) => HttpResponse::Ok().body(serde_json::to_string(&signatures).unwrap()),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/v1/signatures/hash/{kind}/{input}/{page}")]
async fn signatures_by_hash(path: web::Path<ContentPath>, state: web::Data<AppState>) -> impl Responder {
    if path.input.trim().len() != 8 && path.input.trim().len() != 64 {
        return HttpResponse::BadRequest().body("Query must have 8 or 64 characters");
    }

    let kind = match path.kind {
        Kind::All => None,
        Kind::Function => Some(SignatureKind::Function),
        Kind::Event => Some(SignatureKind::Event),
        Kind::Error => Some(SignatureKind::Error),
    };

    match state.dbc.rest().signature_where_hash_starts_with(&path.input, kind, path.page) {
        Some(signatures) => HttpResponse::Ok().body(serde_json::to_string(&signatures).unwrap()),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/v1/sources/github/{kind}/{signature_id}/{page}")]
async fn sources_github(path: web::Path<SourcePath>, state: web::Data<AppState>) -> impl Responder {
    let kind = match path.kind {
        Kind::All => None,
        Kind::Function => Some(SignatureKind::Function),
        Kind::Event => Some(SignatureKind::Event),
        Kind::Error => Some(SignatureKind::Error),
    };

    match state.dbc.rest().sources_github(path.signature_id, kind, path.page) {
        Some(signatures) => HttpResponse::Ok().body(serde_json::to_string(&signatures).unwrap()),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/v1/sources/etherscan/{kind}/{signature_id}/{page}")]
async fn sources_etherscan(path: web::Path<SourcePath>, state: web::Data<AppState>) -> impl Responder {
    let kind = match path.kind {
        Kind::All => None,
        Kind::Function => Some(SignatureKind::Function),
        Kind::Event => Some(SignatureKind::Event),
        Kind::Error => Some(SignatureKind::Error),
    };

    match state.dbc.rest().sources_etherscan(path.signature_id, kind, path.page) {
        Some(signatures) => HttpResponse::Ok().body(serde_json::to_string(&signatures).unwrap()),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/v1/statistics")]
async fn statistics(state: web::Data<AppState>) -> impl Responder {
    #[derive(Serialize)]
    struct Statistics {
        statistics_various_signature_counts: ViewSignatureCountStatistics,
        statistics_signature_insert_rate: Vec<ViewSignatureInsertRate>,
        statistics_signature_kind_distribution: Vec<ViewSignatureKindDistribution>,
        statistics_signatures_popular_on_github: Vec<ViewSignaturesPopularOnGithub>,
    }

    HttpResponse::Ok().body(
        serde_json::to_string(&Statistics {
            statistics_various_signature_counts: state.dbc.rest().statistics_various_signature_counts(),
            statistics_signature_insert_rate: state.dbc.rest().statistics_signature_insert_rate(),
            statistics_signature_kind_distribution: state.dbc.rest().statistics_signature_kind_distribution(),
            statistics_signatures_popular_on_github: state
                .dbc
                .rest()
                .statistics_signatures_popular_on_github(),
        })
        .unwrap(),
    )
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let state = web::Data::new(AppState {
        dbc: DatabaseClientPooled::new().unwrap(),
    });

    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file("/etc/letsencrypt/live/api.etherface.io/privkey.pem", SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file("/etc/letsencrypt/live/api.etherface.io/fullchain.pem").unwrap();

    HttpServer::new(move || {
        App::new()
            // Clone the state here as otherwise each worker thread would create one state /
            // DatabaseClientPooled struct yielding really bad performance
            .app_data(state.clone())
            .service(signatures_by_text)
            .service(signatures_by_hash)
            .service(sources_github)
            .service(sources_etherscan)
            .service(statistics)
            .wrap(Cors::permissive())
            .wrap(Logger::new("[%t] %U, %r"))
    })
    // .bind(("65.21.54.11", 80))?
    .bind_openssl("65.21.54.11:443", builder)?
    .run()
    .await
}
