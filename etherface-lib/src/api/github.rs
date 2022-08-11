//! GitHub API client.
//!
//! Currently covers only the necessary `/user`, `/repositories` and `/search` (sub-)endpoints needed for
//! crawling and finding Solidity repositories.

pub mod handler;
mod page;
pub(crate) mod token;

use super::GithubResponseHandler;
use super::RequestHandler;
use crate::api::github::handler::repositories::RepoHandler;
use crate::api::github::handler::search::SearchHandler;
use crate::api::github::handler::user::UserHandler;
use crate::error::Error;
use reqwest::blocking::Response;
use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest::Url;

const GITHUB_BASE_URL: &str = "https://api.github.com";
const GITHUB_RATELIMIT_URL: &str = "https://api.github.com/rate_limit";

/// See https://docs.github.com/en/rest/overview/resources-in-the-rest-api#current-version
const HEADER_API_VERSION: &str = "application/vnd.github.v3+json";

/// See https://docs.github.com/en/rest/overview/resources-in-the-rest-api#user-agent-required
const HEADER_USER_AGENT: &str = "Etherface";

pub struct GithubClient {
    request_handler: RequestHandler,
}

impl GithubClient {
    /// Returns a new GitHub API client.
    pub fn new() -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HEADER_API_VERSION.parse().unwrap());
        headers.insert(header::USER_AGENT, HEADER_USER_AGENT.parse().unwrap());

        Ok(GithubClient {
            request_handler: RequestHandler::new_github()?,
        })
    }
}

/// API methods
impl GithubClient {
    /// Returns a handler for the `/user/{id}/` endpoint.
    pub fn user(&self, id: i32) -> UserHandler {
        UserHandler::new(self, id)
    }

    /// Returns a handler for the `/repositories/{id}/` endpoint.
    pub fn repos(&self, id: i32) -> RepoHandler {
        RepoHandler::new(self, id)
    }

    /// Returns a handler for the `/search` endpoint.
    pub fn search(&self) -> SearchHandler {
        SearchHandler::new(self)
    }
}

/// HTTP methods
impl GithubClient {
    fn execute(&self, path: &str) -> Result<Response, Error> {
        self.request_handler.execute_resp::<GithubResponseHandler>(&to_absolute_url(path))
    }

    fn execute_with_header(&self, path: &str, header: (&str, &str)) -> Result<Response, Error> {
        self.request_handler.execute_resp_header::<GithubResponseHandler>(&to_absolute_url(path), header)
    }
}

#[inline]
fn to_absolute_url(path: &str) -> String {
    if let Err(url::ParseError::RelativeUrlWithoutBase) = Url::parse(path) {
        return format!("{}/{}", GITHUB_BASE_URL, path);
    }

    path.to_string() // Already an absolute URL, return as is
}
