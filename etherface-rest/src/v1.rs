use actix_web::get;
use actix_web::web;
use actix_web::HttpResponse;
use actix_web::Responder;
use etherface_lib::database::handler::DatabaseClientPooled;
use etherface_lib::model::views::ViewSignatureCountStatistics;
use etherface_lib::model::views::ViewSignatureInsertRate;
use etherface_lib::model::views::ViewSignatureKindDistribution;
use etherface_lib::model::views::ViewSignaturesPopularOnGithub;
use etherface_lib::model::SignatureKind;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    All,
    Function,
    Event,
    Error,
}

#[derive(Deserialize)]
pub struct ContentPath {
    input: String,
    kind: Kind,
    page: i64,
}

#[derive(Deserialize)]
pub struct SourcePath {
    signature_id: i32,
    kind: Kind,
    page: i64,
}

pub struct AppState {
    pub dbc: DatabaseClientPooled,
}

#[inline]
fn is_valid_page_index(index: i64) -> bool {
    index >= 1
}

#[inline]
fn query_kind_to_signaturekind(kind: &Kind) -> Option<SignatureKind> {
    match kind {
        Kind::All => None,
        Kind::Function => Some(SignatureKind::Function),
        Kind::Event => Some(SignatureKind::Event),
        Kind::Error => Some(SignatureKind::Error),
    }
}

#[get("/signatures/text/{kind}/{input}/{page}")]
async fn signatures_by_text(path: web::Path<ContentPath>, state: web::Data<AppState>) -> impl Responder {
    if !is_valid_page_index(path.page) {
        return HttpResponse::BadRequest().body("Page index must be >= 1");
    }

    let input_trimmed = path.input.trim();
    if input_trimmed.len() < 3 {
        return HttpResponse::BadRequest().body("Query must have at least 3 characters");
    }

    let kind = query_kind_to_signaturekind(&path.kind);
    match state.dbc.rest().signatures_where_text_starts_with(&input_trimmed, kind, path.page) {
        Some(signatures) => HttpResponse::Ok().body(serde_json::to_string(&signatures).unwrap()),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/signatures/hash/{kind}/{input}/{page}")]
async fn signatures_by_hash(path: web::Path<ContentPath>, state: web::Data<AppState>) -> impl Responder {
    if !is_valid_page_index(path.page) {
        return HttpResponse::BadRequest().body("Page index must be >= 1");
    }

    let mut input_trimmed = path.input.trim();
    if input_trimmed.starts_with("0x") {
        input_trimmed = &input_trimmed[2..];
    }

    if input_trimmed.len() != 8 && input_trimmed.len() != 64 {
        return HttpResponse::BadRequest().body("Query must have 8 or 64 characters");
    }

    let kind = query_kind_to_signaturekind(&path.kind);
    match state.dbc.rest().signature_where_hash_starts_with(&input_trimmed, kind, path.page) {
        Some(signatures) => HttpResponse::Ok().body(serde_json::to_string(&signatures).unwrap()),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/sources/github/{kind}/{signature_id}/{page}")]
async fn sources_github(path: web::Path<SourcePath>, state: web::Data<AppState>) -> impl Responder {
    if !is_valid_page_index(path.page) {
        return HttpResponse::BadRequest().body("Page index must be >= 1");
    }

    let kind = query_kind_to_signaturekind(&path.kind);
    match state.dbc.rest().sources_github(path.signature_id, kind, path.page) {
        Some(signatures) => HttpResponse::Ok().body(serde_json::to_string(&signatures).unwrap()),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/sources/etherscan/{kind}/{signature_id}/{page}")]
async fn sources_etherscan(path: web::Path<SourcePath>, state: web::Data<AppState>) -> impl Responder {
    if !is_valid_page_index(path.page) {
        return HttpResponse::BadRequest().body("Page index must be >= 1");
    }

    let kind = query_kind_to_signaturekind(&path.kind);
    match state.dbc.rest().sources_etherscan(path.signature_id, kind, path.page) {
        Some(signatures) => HttpResponse::Ok().body(serde_json::to_string(&signatures).unwrap()),
        None => HttpResponse::NotFound().finish(),
    }
}

#[get("/statistics")]
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