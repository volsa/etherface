use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // GitHub Errors
    #[error("Failed to retrieve resource '{0}', likely removed from GitHub")]
    GithubResourceUnavailable(String),

    #[error("Failed to find valid tokens in the token pool, either they're invalid or not present")]
    GithubTokenPoolEmpty,

    #[error("Failed to request data, token invalid")]
    GithubTokenInvalid,

    #[error("Failed to deserialize JSON input; {0}")]
    DeserializeError(#[from] serde_json::Error),

    #[error("{0}")]
    ResponseHandlerInvalidFunctionCall(String),

    // Etherscan Errors
    #[error("Invalid Etherscan token '{0}'")]
    EtherscanInvalidToken(String),

    #[error("Failed to deserialize Etherscan JSON response '{0}'; {1}")]
    EtherscanDeserializeJsonResponse(String, String),

    // HTTP Errors
    #[error("Failed to initialize HTTP client; {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Failed to send HTTP request; {0}")]
    HttpRequest(#[source] reqwest::Error),

    // Config Errors
    #[error("Failed to read .env file; {0}")]
    ConfigRead(#[from] dotenv::Error),

    #[error("Environment variable '{0}' does not exist; {1}")]
    ConfigReadNonExistantEnvironmentVariable(&'static str, #[source] std::env::VarError),

    #[error("Environment variable '{0}' is empty")]
    ConfigReadEmptyEnvironmentVariable(&'static str),

    // Misc Errors
    #[error("Failed to read environment variable '{1}'; {0}")]
    EnvironmentVariableRead(#[source] std::env::VarError, &'static str),

    #[error("Failed to connect to database; {0}")]
    DatabaseConnect(#[from] diesel::result::ConnectionError),

    // Parser / Deserializer
    #[error("Failed to deserialize content, invalid ABI?")]
    ParseAbi(#[source] serde_json::Error),

    #[error("Failed to parse pragma version; {0}")]
    ParsePragma(String),

    #[error("Aborting crawling process, one or more background events disconnected from channel")]
    CrawlerChannelDisconnected,
}
