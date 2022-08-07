use crate::error::Error;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub github: GithubConfig,
    pub database: DatabaseConfig,
    pub etherscan: EtherscanConfig,
}

#[derive(Deserialize)]
pub struct GithubConfig {
    pub tokens: Vec<String>,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub dbname: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct EtherscanConfig {
    pub token: String,
}

const ENV_CONFIG_PATH: &str = "ETHERFACE_CONFIG_PATH";
const DEFAULT_CONFIG_PATH: &str = "./etherface.toml";

impl Config {
    pub fn new() -> Result<Self, Error> {
        let config_path = match Path::new(DEFAULT_CONFIG_PATH).exists() {
            true => DEFAULT_CONFIG_PATH.to_string(),
            false => std::env::var(ENV_CONFIG_PATH)
                .map_err(|err| Error::EnvironmentVariableRead(err, ENV_CONFIG_PATH))?,
        };

        let content = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;

        if config.github.tokens.is_empty() {
            return Err(Error::GithubTokenPoolEmpty);
        }

        Ok(config)
    }

    pub fn get_database_url(&self) -> String {
        format!(
            "postgres://{username}:{password}@{host}/{dbname}",
            username = self.database.username,
            password = self.database.password,
            host = self.database.host,
            dbname = self.database.dbname,
        )
    }
}
