use crate::database::pagination::Paginate;
use crate::model::views::ViewSignatureCountStatistics;
use crate::model::views::ViewSignatureInsertRate;
use crate::model::views::ViewSignatureKindDistribution;
use crate::model::views::ViewSignaturesPopularOnGithub;
use crate::model::EtherscanContract;
use crate::model::GithubRepositoryDatabase;
use crate::model::Signature;
use crate::model::SignatureKind;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::sql_query;
use diesel::PgConnection;
use serde::Serialize;

#[derive(Serialize)]
pub struct RestResponse<T> {
    pub total_pages: i64,
    pub total_items: i64,
    pub items: T,
}

pub struct RestHandler<'a> {
    connection: &'a Pool<ConnectionManager<PgConnection>>,
}

type Response<T> = Option<RestResponse<Vec<T>>>;

impl<'a> RestHandler<'a> {
    pub fn new(connection: &'a Pool<ConnectionManager<PgConnection>>) -> Self {
        RestHandler { connection }
    }

    pub fn signatures_where_text_starts_with(
        &self,
        entity_str: &str,
        entity_kind: Option<SignatureKind>,
        page: i64,
    ) -> Response<Signature> {
        use crate::database::schema::mapping_signature_kind;
        use crate::database::schema::signature;
        use crate::database::schema::signature::dsl::*;
        // use crate::database::schema::mapping_signature_kind::dsl::*;

        let (items, total_items, total_pages) = match entity_kind {
            Some(entity_kind) => {
                let query = signature
                    .inner_join(mapping_signature_kind::table)
                    .filter(
                        signature::text
                            .like(format!("{entity_str}%"))
                            .and(signature::is_valid.eq(true))
                            .and(mapping_signature_kind::kind.eq(entity_kind)),
                    )
                    .order_by(signature::id.asc()) // TODO: find a better way to sort?
                    .select(signature::all_columns)
                    .paginate(page);

                println!("{}", diesel::debug_query::<_, _>(&query).to_string());

                query.load_and_count_pages::<Signature>(&mut self.connection.get().unwrap()).unwrap()
            }

            None => {
                let query = signature
                    .filter(signature::text.like(format!("{entity_str}%")).and(signature::is_valid.eq(true)))
                    .order_by(signature::id.asc()) // TODO: find a better way to sort?
                    .select(signature::all_columns)
                    .paginate(page);

                query.load_and_count_pages::<Signature>(&mut self.connection.get().unwrap()).unwrap()
            }
        };

        match items.len() {
            0 => None,
            _ => Some(RestResponse {
                items,
                total_items,
                total_pages,
            }),
        }
    }

    pub fn signature_where_hash_starts_with(
        &self,
        entity_str: &str,
        entity_kind: Option<SignatureKind>,
        page: i64,
    ) -> Response<Signature> {
        use crate::database::schema::mapping_signature_kind;
        // use crate::database::schema::mapping_signature_kind::dsl::*;
        use crate::database::schema::signature;
        use crate::database::schema::signature::dsl::*;

        let (items, total_items, total_pages) = match entity_kind {
            Some(entity_kind) => {
                let query = signature
                    .inner_join(mapping_signature_kind::table)
                    .filter(
                        signature::hash
                            .like(format!("{entity_str}%"))
                            .and(signature::is_valid.eq(true))
                            .and(mapping_signature_kind::kind.eq(entity_kind)),
                    )
                    .order_by(signature::id.asc()) // TODO: find a better way to sort?
                    .select(signature::all_columns)
                    .paginate(page);

                query.load_and_count_pages::<Signature>(&mut self.connection.get().unwrap()).unwrap()
            }

            None => {
                let query = signature
                    .filter(signature::hash.like(format!("{entity_str}%")).and(signature::is_valid.eq(true)))
                    .order_by(signature::id.asc()) // TODO: find a better way to sort?
                    .select(signature::all_columns)
                    .paginate(page);

                query.load_and_count_pages::<Signature>(&mut self.connection.get().unwrap()).unwrap()
            }
        };

        match items.len() {
            0 => None,
            _ => Some(RestResponse {
                items,
                total_items,
                total_pages,
            }),
        }
    }

    pub fn sources_github(
        &self,
        entity_id: i32,
        entity_kind: Option<SignatureKind>,
        page: i64,
    ) -> Response<GithubRepositoryDatabase> {
        use crate::database::schema::github_repository;
        use crate::database::schema::github_repository::dsl::*;
        use crate::database::schema::mapping_signature_github;
        // use crate::database::schema::mapping_signature_github::dsl::*;

        let (items, total_items, total_pages) = match entity_kind {
            Some(entity_kind) => {
                let query = github_repository
                    .inner_join(mapping_signature_github::table)
                    .filter(
                        mapping_signature_github::signature_id
                            .eq(entity_id)
                            .and(mapping_signature_github::kind.eq(entity_kind))
                            .and(github_repository::fork.eq(false)),
                    )
                    .order_by(github_repository::stargazers_count.desc())
                    .distinct_on((github_repository::id, github_repository::stargazers_count))
                    .select(github_repository::all_columns)
                    .paginate(page);

                query
                    .load_and_count_pages::<GithubRepositoryDatabase>(&mut self.connection.get().unwrap())
                    .unwrap()
            }

            None => {
                let query = github_repository
                    .inner_join(mapping_signature_github::table)
                    .filter(
                        mapping_signature_github::signature_id
                            .eq(entity_id)
                            .and(github_repository::fork.eq(false)),
                    )
                    .order_by(github_repository::stargazers_count.desc())
                    .distinct_on((github_repository::id, github_repository::stargazers_count))
                    .select(github_repository::all_columns)
                    .paginate(page);

                query
                    .load_and_count_pages::<GithubRepositoryDatabase>(&mut self.connection.get().unwrap())
                    .unwrap()
            }
        };

        match items.len() {
            0 => None,
            _ => Some(RestResponse {
                items,
                total_items,
                total_pages,
            }),
        }
    }

    pub fn sources_etherscan(
        &self,
        entity_id: i32,
        entity_kind: Option<SignatureKind>,
        page: i64,
    ) -> Response<EtherscanContract> {
        use crate::database::schema::etherscan_contract;
        use crate::database::schema::etherscan_contract::dsl::*;
        use crate::database::schema::mapping_signature_etherscan;
        // use crate::database::schema::mapping_signature_github::dsl::*;

        let (items, total_items, total_pages) = match entity_kind {
            Some(entity_kind) => {
                let query = etherscan_contract
                    .inner_join(mapping_signature_etherscan::table)
                    .filter(
                        mapping_signature_etherscan::signature_id
                            .eq(entity_id)
                            .and(mapping_signature_etherscan::kind.eq(entity_kind)),
                    )
                    .order_by(etherscan_contract::added_at.desc())
                    .distinct_on((etherscan_contract::id, etherscan_contract::added_at))
                    .select(etherscan_contract::all_columns)
                    .paginate(page);

                query.load_and_count_pages::<EtherscanContract>(&mut self.connection.get().unwrap()).unwrap()
            }
            None => {
                let query = etherscan_contract
                    .inner_join(mapping_signature_etherscan::table)
                    .filter(mapping_signature_etherscan::signature_id.eq(entity_id))
                    .order_by(etherscan_contract::added_at.desc())
                    .distinct_on((etherscan_contract::id, etherscan_contract::added_at))
                    .select(etherscan_contract::all_columns)
                    .paginate(page);

                query.load_and_count_pages::<EtherscanContract>(&mut self.connection.get().unwrap()).unwrap()
            }
        };

        match items.len() {
            0 => None,
            _ => Some(RestResponse {
                items,
                total_items,
                total_pages,
            }),
        }
    }

    pub fn statistics_signature_insert_rate(&self) -> Vec<ViewSignatureInsertRate> {
        sql_query("SELECT date, count FROM view_signature_insert_rate")
            .get_results(&self.connection.get().unwrap())
            .unwrap()
    }

    pub fn statistics_various_signature_counts(&self) -> ViewSignatureCountStatistics {
        sql_query("SELECT signature_count, signature_count_github, signature_count_etherscan, signature_count_fourbyte, average_daily_signature_insert_rate_last_week, average_daily_signature_insert_rate_week_before_last FROM view_signature_count_statistics")
            .get_result(&self.connection.get().unwrap())
            .unwrap()
    }

    pub fn statistics_signatures_popular_on_github(&self) -> Vec<ViewSignaturesPopularOnGithub> {
        sql_query("SELECT text, count FROM view_signatures_popular_on_github")
            .get_results(&self.connection.get().unwrap())
            .unwrap()
    }

    pub fn statistics_signature_kind_distribution(&self) -> Vec<ViewSignatureKindDistribution> {
        sql_query("SELECT kind, count FROM view_signature_kind_distribution")
            .get_results(&self.connection.get().unwrap())
            .unwrap()
    }
}
