//! Structs that are both used in the GitHub API as well as the Database schema / bindings.

#![allow(clippy::extra_unused_lifetimes)] // Clippy complains about the Insertable proc-macro

use crate::database::schema::*;
use chrono::DateTime;
use chrono::Utc;
use diesel::Insertable;
use diesel::Queryable;
use diesel_derive_enum::DbEnum;
use serde::Deserialize;
use serde::Serialize;
use sha3::Digest;
use sha3::Keccak256;
use std::str::FromStr;

#[derive(Queryable, Insertable)]
#[table_name = "github_crawler_metadata"]
pub struct GithubCrawlerMetadata {
    pub id: i32,
    pub last_user_check: DateTime<Utc>,
    pub last_repository_check: DateTime<Utc>,
    pub last_repository_search: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GithubUser {
    pub id: i32,
    pub login: String,
    pub html_url: String,
    pub public_repos: Option<i32>, // Needs to be an Option because not every response has a value for it
                                   // See for example https://api.github.com/repos/ethereum/fe/stargazers
}

impl GithubUser {
    pub fn to_insertable(&self) -> GithubUserDatabase {
        GithubUserDatabase {
            id: self.id,
            login: self.login.clone(),
            html_url: self.html_url.clone(),

            is_deleted: false, // Initially always false (as we can query it) and only updated if the GitHub API fails to retrieve the user
            visited_at: None,
            added_at: Utc::now(),
        }
    }
}

#[derive(Queryable, Insertable)]
#[table_name = "github_user"]
pub struct GithubUserDatabase {
    pub id: i32,
    pub login: String,
    pub html_url: String,
    pub is_deleted: bool,
    pub added_at: DateTime<Utc>,
    pub visited_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GithubRepository {
    pub id: i32,
    pub name: String,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: i32,
    pub size: i32,
    pub fork: bool,

    #[serde(rename = "source")]
    pub fork_parent: Option<Box<GithubRepository>>,
    pub created_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub owner: GithubUser,
}

#[derive(Queryable, Insertable, Deserialize, Serialize, QueryableByName)]
#[table_name = "github_repository"]
pub struct GithubRepositoryDatabase {
    pub id: i32,
    pub owner_id: i32,
    pub name: String,
    pub html_url: String,
    pub language: Option<String>,
    pub stargazers_count: i32,
    pub size: i32,
    pub fork: bool,
    pub created_at: DateTime<Utc>,
    pub pushed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    pub scraped_at: Option<DateTime<Utc>>,
    pub visited_at: Option<DateTime<Utc>>,
    pub added_at: DateTime<Utc>,

    pub solidity_ratio: Option<f32>,
    pub is_deleted: bool,
    pub found_by_crawling: bool,
}

impl GithubRepository {
    pub fn to_insertable(&self, solidity_ratio: Option<f32>, by_crawling: bool) -> GithubRepositoryDatabase {
        // XXX: This isn't ideal because there are multiple copy semantics but it doesn't make sense
        // to create a RepositoryDatabaseInsert<'a> struct because it's 1:1 the same as RepositoryDatabase
        GithubRepositoryDatabase {
            id: self.id,
            owner_id: self.owner.id,
            name: self.name.clone(),
            html_url: self.html_url.clone(),
            language: self.language.clone(),
            stargazers_count: self.stargazers_count,
            size: self.size,
            fork: self.fork,
            created_at: self.created_at,
            pushed_at: self.pushed_at,
            updated_at: self.updated_at,
            is_deleted: false,

            solidity_ratio,
            found_by_crawling: by_crawling,

            // Both fields are initially None and will be updated once the crawler / scraper visited them
            visited_at: None,
            scraped_at: None,
            added_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Queryable)]
pub struct EtherscanContract {
    pub id: i32,
    pub address: String,
    pub name: String,
    pub compiler: String,
    pub compiler_version: String,
    pub url: String,
    pub scraped_at: Option<DateTime<Utc>>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[table_name = "etherscan_contract"]
pub struct EtherscanContractInsert<'a> {
    pub address: &'a str,
    pub name: &'a str,
    pub compiler: &'a str,
    pub compiler_version: &'a str,
    pub url: &'a str,
    pub added_at: &'a DateTime<Utc>,
}

impl EtherscanContract {
    pub fn to_insertable(&self) -> EtherscanContractInsert {
        EtherscanContractInsert {
            address: &self.address,
            name: &self.name,
            compiler: &self.compiler,
            compiler_version: &self.compiler_version,
            url: &self.url,
            added_at: &self.added_at,
        }
    }
}

#[derive(Queryable, Serialize, Debug)]
pub struct Signature {
    pub id: i32,
    pub text: String,
    pub hash: String,
    pub is_valid: bool,
    pub added_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[table_name = "signature"]
pub struct SignatureInsert<'a> {
    pub text: &'a str,
    pub hash: &'a str,
    pub is_valid: bool,
    pub added_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct SignatureWithMetadata {
    pub text: String,
    pub hash: String,
    pub kind: SignatureKind,
    pub is_valid: bool,
}

#[derive(Queryable, Insertable)]
#[table_name = "mapping_signature_github"]
pub struct MappingSignatureGithub {
    pub signature_id: i32,
    pub repository_id: i32,
    pub kind: SignatureKind,
    pub added_at: DateTime<Utc>,
}

#[derive(Queryable, Insertable)]
#[table_name = "mapping_signature_etherscan"]
pub struct MappingSignatureEtherscan {
    pub signature_id: i32,
    pub contract_id: i32,
    pub kind: SignatureKind,
    pub added_at: DateTime<Utc>,
}

#[derive(Queryable, Insertable)]
#[table_name = "mapping_signature_fourbyte"]
pub struct MappingSignatureFourbyte {
    pub signature_id: i32,
    pub kind: SignatureKind,
    pub added_at: DateTime<Utc>,
}

#[derive(Queryable, Insertable)]
#[table_name = "mapping_signature_kind"]
pub struct MappingSignatureKind {
    pub signature_id: i32,
    pub kind: SignatureKind,
}

impl SignatureWithMetadata {
    pub fn new(text: String, kind: SignatureKind, is_valid: bool) -> Self {
        let hash = format!("{:x}", Keccak256::digest(&text));

        Self {
            text,
            hash,
            kind,
            is_valid,
        }
    }

    pub fn to_insertable(&self) -> SignatureInsert {
        SignatureInsert {
            text: &self.text,
            hash: &self.hash,
            is_valid: self.is_valid,
            added_at: Utc::now(),
        }
    }
}

#[derive(Serialize, Deserialize, DbEnum, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
#[DieselType = "Signature_kind"]
pub enum SignatureKind {
    Function,
    Event,
    Error,
    Constructor,
    Fallback,
    Receive,
}

impl FromStr for SignatureKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "function" => Ok(SignatureKind::Function),
            "event" => Ok(SignatureKind::Event),
            "error" => Ok(SignatureKind::Error),
            "constructor" => Ok(SignatureKind::Constructor),
            "fallback" => Ok(SignatureKind::Fallback),
            "receive" => Ok(SignatureKind::Receive),

            // The function should never return an error as long as Solidity does not introduce a new
            // interface kind which we have not yet covered in our above pattern matching.
            _ => Err(()),
        }
    }
}

/// Materialized Views introduced with the `2022-08-01-201536_create_materialized_views` migration
pub mod views {
    use chrono::NaiveDate;
    use diesel::sql_types::BigInt;
    use diesel::sql_types::Date;
    use diesel::sql_types::Text;
    use diesel::sql_types::Nullable;
    use diesel::Queryable;
    use diesel::QueryableByName;
    use serde::Serialize;

    #[derive(Queryable, QueryableByName, Serialize)]
    pub struct ViewSignatureInsertRate {
        #[sql_type = "Date"]
        date: NaiveDate,

        #[sql_type = "BigInt"]
        count: i64,
    }

    #[derive(Queryable, QueryableByName, Serialize)]
    pub struct ViewSignaturesPopularOnGithub {
        #[sql_type = "Text"]
        text: String,

        #[sql_type = "BigInt"]
        count: i64,
    }

    #[derive(Queryable, QueryableByName, Serialize)]
    pub struct ViewSignatureCountStatistics {
        #[sql_type = "BigInt"]
        signature_count: i64,

        #[sql_type = "BigInt"]
        signature_count_github: i64,

        #[sql_type = "BigInt"]
        signature_count_etherscan: i64,

        #[sql_type = "BigInt"]
        signature_count_fourbyte: i64,

        #[sql_type = "BigInt"]
        average_daily_signature_insert_rate_last_week: i64,

        #[sql_type = "Nullable<BigInt>"]
        average_daily_signature_insert_rate_week_before_last: Option<i64>, // This can be NULL in the first week
    }

    #[derive(Queryable, QueryableByName, Serialize)]
    pub struct ViewSignatureKindDistribution {
        #[sql_type = "Text"]
        kind: String,

        #[sql_type = "BigInt"]
        count: i64,
    }
}
