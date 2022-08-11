//! Config manager, reading the content of the `.env` file.
//! 
//! Reads all content from `.env` into [`Config`] for all sub-modules to use.

use crate::error::Error;
use dotenv::dotenv;
use std::path::Path;

pub struct Config {
    /// Database URL with the following structure `postgres://username:password@host/database_name`.
    pub database_url: String,

    /// Etherscan API token.
    pub token_etherscan: String,

    /// GitHub API tokens.
    pub tokens_github: Vec<String>,

    /// Etherface REST API address, e.g. <https://api.etherface.io>
    pub rest_address: String,
}

const ENV_VAR_DATABASE_URL: &str = "ETHERFACE_DATABASE_URL";
const ENV_VAR_TOKEN_ETHERSCAN: &str = "ETHERFACE_TOKEN_ETHERSCAN";
const ENV_VAR_TOKENS_GITHUB: &str = "ETHERFACE_TOKENS_GITHUB";
const ENV_VAR_REST_ADDRESS: &str = "ETHERFACE_REST_ADDRESS";

#[inline]
fn read_and_return_env_var(env_var: &'static str) -> Result<String, Error> {
    let res = std::env::var(env_var)
        .map_err(|err| Error::ConfigReadNonExistantEnvironmentVariable(env_var, err))?;

    match res.is_empty() {
        true => Err(Error::ConfigReadEmptyEnvironmentVariable(env_var)),
        false => Ok(res),
    }
}

impl Config {
    /// Returns a new config manager, reading the content of `.env`.
    pub fn new() -> Result<Self, Error> {
        match Path::new(".env").exists() {
            true => dotenv()?,
            false => dotenv::from_filename("../.env")?, // If executed within a sub-directory
        };

        let database_url = read_and_return_env_var(ENV_VAR_DATABASE_URL)?;
        let token_etherscan = read_and_return_env_var(ENV_VAR_TOKEN_ETHERSCAN)?;
        let rest_address = read_and_return_env_var(ENV_VAR_REST_ADDRESS)?;

        let tokens_github = std::env::var(ENV_VAR_TOKENS_GITHUB)
            .map_err(|err| Error::ConfigReadNonExistantEnvironmentVariable(ENV_VAR_TOKENS_GITHUB, err))?
            .split(',')
            .map(str::to_string)
            .collect::<Vec<String>>();

        if tokens_github.is_empty() {
            return Err(Error::ConfigReadEmptyEnvironmentVariable(ENV_VAR_TOKENS_GITHUB));
        }

        Ok(Config {
            database_url,
            tokens_github,
            token_etherscan,
            rest_address,
        })
    }
}
