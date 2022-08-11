//! Database table handlers.
//!
//! All tables can be further inspected in the `migrations/2022-03-06-133006_etherface_database/up.sql` or
//! `schema.rs` file.

pub mod etherscan_contract;
pub mod github_crawler_metadata;
pub mod github_repository;
pub mod github_user;
pub mod mapping_signature_etherscan;
pub mod mapping_signature_fourbyte;
pub mod mapping_signature_github;
pub mod rest;
pub mod signature;

use crate::config::Config;
use crate::database::handler::etherscan_contract::EtherscanContractHandler;
use crate::database::handler::github_crawler_metadata::GithubCrawlerMetadataHandler;
use crate::database::handler::github_repository::GithubRepositoryHandler;
use crate::database::handler::github_user::GithubUserHandler;
use crate::database::handler::mapping_signature_etherscan::MappingSignatureEtherscanHandler;
use crate::database::handler::mapping_signature_fourbyte::MappingSignatureFourbyteHandler;
use crate::database::handler::mapping_signature_github::MappingSignatureGithubHandler;
use crate::database::handler::rest::RestHandler;
use crate::database::handler::signature::SignatureHandler;
use crate::error::Error;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::Connection;
use diesel::PgConnection;

/// Database client, providing all table handlers.
pub struct DatabaseClient {
    connection: PgConnection,
}

/// Same as [`DatabaseClient`] but threaded for the REST API.
pub struct DatabaseClientPooled {
    connection: Pool<ConnectionManager<PgConnection>>,
}

impl DatabaseClientPooled {
    /// Returns a new threaded database client.
    pub fn new() -> Result<Self, Error> {
        // TODO: https://docs.diesel.rs/diesel/r2d2/struct.Builder.html
        let config = Config::new()?;
        let manager = diesel::r2d2::ConnectionManager::<PgConnection>::new(&config.database_url);
        let pool = diesel::r2d2::Pool::builder().build(manager).unwrap();

        Ok(DatabaseClientPooled { connection: pool })
    }

    /// Returns a handler for REST specific purposes.
    pub fn rest(&self) -> RestHandler {
        RestHandler::new(&self.connection)
    }
}

impl DatabaseClient {
    /// Returns a new database client.
    pub fn new() -> Result<Self, Error> {
        let config = Config::new()?;

        Ok(DatabaseClient {
            connection: PgConnection::establish(&config.database_url)?,
        })
    }

    /// Returns a handler for the `github_user` table.
    pub fn github_user(&self) -> GithubUserHandler {
        GithubUserHandler::new(&self.connection)
    }

    /// Returns a handler for the `github_repository` table.
    pub fn github_repository(&self) -> GithubRepositoryHandler {
        GithubRepositoryHandler::new(&self.connection)
    }

    /// Returns a handler for the `etherscan_contract` table.
    pub fn etherscan_contract(&self) -> EtherscanContractHandler {
        EtherscanContractHandler::new(&self.connection)
    }

    /// Returns a handler for the `signature` table.
    pub fn signature(&self) -> SignatureHandler {
        SignatureHandler::new(&self.connection)
    }

    /// Returns a handler for the `mapping_signature_etherscan` table.
    pub fn mapping_signature_etherscan(&self) -> MappingSignatureEtherscanHandler {
        MappingSignatureEtherscanHandler::new(&self.connection)
    }

    /// Returns a handler for the `mapping_signature_fourbyte` table.
    pub fn mapping_signature_fourbyte(&self) -> MappingSignatureFourbyteHandler {
        MappingSignatureFourbyteHandler::new(&self.connection)
    }

    /// Returns a handler for the `mapping_signature_github` table.
    pub fn mapping_signature_github(&self) -> MappingSignatureGithubHandler {
        MappingSignatureGithubHandler::new(&self.connection)
    }

    /// Returns a handler for the `github_crawler_metadata` table.
    pub fn github_crawler_metadata(&self) -> GithubCrawlerMetadataHandler {
        GithubCrawlerMetadataHandler::new(&self.connection)
    }
}
