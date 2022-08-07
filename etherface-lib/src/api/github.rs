mod handler;
mod page;
pub(crate) mod token;

use crate::api::github::handler::repos::RepoHandler;
use crate::api::github::handler::search::SearchHandler;
use crate::api::github::handler::users::UserHandler;
use crate::error::Error;
use reqwest::blocking::Response;
use reqwest::header;
use reqwest::header::HeaderMap;
use reqwest::Url;

use super::GithubResponseHandler;
use super::RequestHandler;

const GITHUB_BASE_URL: &str = "https://api.github.com";
const GITHUB_RATELIMIT_URL: &str = "https://api.github.com/rate_limit";
const HEADER_API_VERSION: &str = "application/vnd.github.v3+json"; // https://docs.github.com/en/rest/overview/resources-in-the-rest-api#current-version
const HEADER_USER_AGENT: &str = "Etherface"; // https://docs.github.com/en/rest/overview/resources-in-the-rest-api#user-agent-required

pub struct GithubClient {
    request_handler: RequestHandler,
}

impl GithubClient {
    pub fn new() -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT, HEADER_API_VERSION.parse().unwrap());
        headers.insert(header::USER_AGENT, HEADER_USER_AGENT.parse().unwrap());

        Ok(GithubClient {
            request_handler: RequestHandler::new_github()?,
        })
    }
}

// API methods
impl GithubClient {
    pub fn user(&self, id: i32) -> UserHandler {
        UserHandler::new(self, id)
    }

    pub fn repos(&self, id: i32) -> RepoHandler {
        RepoHandler::new(self, id)
    }

    pub fn search(&self) -> SearchHandler {
        SearchHandler::new(self)
    }
}

// HTTP methods
impl GithubClient {
    fn execute(&self, path: &str) -> Result<Response, Error> {
        self.request_handler.execute_resp::<GithubResponseHandler>(&to_absolute_url(path))
    }

    fn execute_with_header(&self, path: &str, header: (&str, &str)) -> Result<Response, Error> {
        self.request_handler.execute_resp_header::<GithubResponseHandler>(&to_absolute_url(path), header)
    }
}

// TODO: move to `RequestHandler`
fn to_absolute_url(path: &str) -> String {
    if let Err(url::ParseError::RelativeUrlWithoutBase) = Url::parse(path) {
        return format!("{}/{}", GITHUB_BASE_URL, path);
    }

    path.to_string() // Already an absolute URL, return as is
}

pub fn parse_error_message(response: Response) -> String {
    let content = response.text().unwrap();

    if content.is_empty() {
        return "n/a".to_string();
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
        let json: serde_json::Value = serde_json::from_value(value).unwrap();
        return json.get("message").unwrap().to_string();
    }

    content
}
