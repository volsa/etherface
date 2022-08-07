use std::path::Path;

use crate::error::Error;
use dotenv::dotenv;

pub struct Config {
    pub database_url: String,
    pub token_etherscan: String,
    pub tokens_github: Vec<String>,
    pub rest_address: String,
}

const ENV_VAR_DATABASE_URL: &str = "ETHERFACE_DATABASE_URL";
const ENV_VAR_TOKEN_ETHERSCAN: &str = "ETHERFACE_TOKEN_ETHERSCAN";
const ENV_VAR_TOKENS_GITHUB: &str = "ETHERFACE_TOKENS_GITHUB";
const ENV_VAR_REST_ADDRESS: &str = "ETHERFACE_REST_ADDRESS";

impl Config {
    pub fn new() -> Result<Self, Error> {
        match Path::new(".env").exists() {
            true => dotenv()?,
            false => dotenv::from_filename("../.env")?, // If executed within a sub-directory
        };

        let database_url = std::env::var(ENV_VAR_DATABASE_URL)
            .map_err(|err| Error::ConfigReadNonExistantEnvironmentVariable(ENV_VAR_DATABASE_URL, err))?;

        let token_etherscan = std::env::var(ENV_VAR_TOKEN_ETHERSCAN)
            .map_err(|err| Error::ConfigReadNonExistantEnvironmentVariable(ENV_VAR_TOKEN_ETHERSCAN, err))?;

        let tokens_github = std::env::var(ENV_VAR_TOKENS_GITHUB)
            .map_err(|err| Error::ConfigReadNonExistantEnvironmentVariable(ENV_VAR_TOKENS_GITHUB, err))?
            .split(',')
            .map(str::to_string)
            .collect::<Vec<String>>();

        let rest_address = std::env::var(ENV_VAR_TOKEN_ETHERSCAN)
            .map_err(|err| Error::ConfigReadNonExistantEnvironmentVariable(ENV_VAR_TOKEN_ETHERSCAN, err))?;

        if database_url.is_empty() {
            return Err(Error::ConfigReadEmptyEnvironmentVariable(ENV_VAR_DATABASE_URL));
        }

        if token_etherscan.is_empty() {
            return Err(Error::ConfigReadEmptyEnvironmentVariable(ENV_VAR_TOKEN_ETHERSCAN));
        }

        if tokens_github.is_empty() {
            return Err(Error::ConfigReadEmptyEnvironmentVariable(ENV_VAR_TOKENS_GITHUB));
        }

        if rest_address.is_empty() {
            return Err(Error::ConfigReadEmptyEnvironmentVariable(ENV_VAR_REST_ADDRESS));
        }

        Ok(Config {
            database_url,
            tokens_github,
            token_etherscan,
            rest_address,
        })
    }
}
