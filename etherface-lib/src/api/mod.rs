//! GitHub, Etherscan and 4Byte API clients.

use crate::api::github::token::TokenManager;
use crate::error::Error;
use log::debug;
use reqwest::blocking::Client;
use reqwest::blocking::RequestBuilder;
use reqwest::blocking::Response;
use reqwest::header;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::cell::RefCell;

pub mod etherscan;
pub mod fourbyte;
pub mod github;

struct RequestHandler {
    client: Client,
    github_tokenmanager: Option<RefCell<TokenManager>>,
}

const GITHUB_USER_AGENT: &str = "Etherface";

/// Handler responsible for sites which don't need any special error handling
struct GenericResponseHandler;

/// Handler responsible for Ethersca
struct EtherscanResponseHandler;
struct GithubResponseHandler;
struct TokenManagerResponseHandler;

///
trait ResponseHandler {
    /// Prepares a request by i.e. setting it's headers or query parameters.
    fn prepare(request_handler: &RequestHandler, url: &str) -> RequestBuilder {
        request_handler.client.get(url)
    }

    /// Given a response different error status codes are handled.
    fn process(response: Response) -> Result<ResponseHandlerResult, Error>;
}

///
enum ResponseHandlerResult {
    Ok(Content),
    Retry(String),
    RetryWithAction(Action),
    RetryWithCustomSleepDuration(u64),
}

///
enum Content {
    Response(Response),
    Text(String),
}

///
enum Action {
    GithubCleanup,
    GithubRefresh,
}

impl RequestHandler {
    pub fn new() -> Self {
        RequestHandler {
            client: Client::default(),
            github_tokenmanager: None,
        }
    }

    pub fn new_github() -> Result<Self, Error> {
        Ok(RequestHandler {
            client: Client::default(),
            github_tokenmanager: Some(RefCell::new(TokenManager::new()?)),
        })
    }

    #[inline]
    fn execute<T: ResponseHandler>(
        &self,
        url: &str,
        header: Option<(&str, &str)>,
        token: Option<&str>,
    ) -> Result<Content, Error> {
        let mut retries = 0;
        let mut retries_valid = 1;

        loop {
            let mut request = T::prepare(self, url);

            if let Some(header) = header {
                request = request.header(header.0, header.1);
            }

            if let Some(token) = token {
                request = request.bearer_auth(token);
            }

            match request.send() {
                Ok(response) => match T::process(response)? {
                    ResponseHandlerResult::Ok(body) => return Ok(body),

                    ResponseHandlerResult::Retry(why) => {
                        debug!("Retrying because of '{why}' ({url})");
                        if retries_valid < 10 {
                            retries_valid += 1;
                        }
                    }

                    ResponseHandlerResult::RetryWithAction(action) => match action {
                        Action::GithubCleanup => {
                            self.github_tokenmanager.as_ref().unwrap().borrow_mut().cleanup()?;
                            continue;
                        }

                        Action::GithubRefresh => {
                            self.github_tokenmanager.as_ref().unwrap().borrow_mut().refresh()?;
                            continue;
                        }
                    },

                    ResponseHandlerResult::RetryWithCustomSleepDuration(duration) => {
                        std::thread::sleep(std::time::Duration::from_secs(duration));
                        continue;
                    }
                },

                Err(why) => {
                    retries += 1;

                    // Return an error if after N retries the reqwest crate is unable to send a request.
                    if retries == 5 {
                        return Err(Error::HttpRequest(why));
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(5 * retries_valid));
        }
    }

    pub fn execute_resp<T: ResponseHandler>(&self, url: &str) -> Result<Response, Error> {
        match self.execute::<T>(url, None, None)? {
            Content::Response(response) => Ok(response),

            _ => Err(Error::ResponseHandlerInvalidFunctionCall(
                "You probably meant to call one of the `execute_deser` functions".to_string(),
            )),
        }
    }

    pub fn execute_resp_header<T: ResponseHandler>(
        &self,
        url: &str,
        header: (&str, &str),
    ) -> Result<Response, Error> {
        match self.execute::<T>(url, Some(header), None)? {
            Content::Response(response) => Ok(response),

            _ => Err(Error::ResponseHandlerInvalidFunctionCall(
                "You probably meant to call one of the `execute_deser` functions".to_string(),
            )),
        }
    }

    pub fn execute_deser<T: ResponseHandler, U: DeserializeOwned>(&self, url: &str) -> Result<U, Error> {
        match self.execute::<T>(url, None, None)? {
            Content::Response(response) => Ok(response.json()?),
            Content::Text(content) => Ok(serde_json::from_str(&content)?),
        }
    }

    pub fn execute_deser_token<T: ResponseHandler, U: DeserializeOwned>(
        &self,
        url: &str,
        token: &str,
    ) -> Result<U, Error> {
        match self.execute::<T>(url, None, Some(token))? {
            Content::Response(response) => Ok(response.json()?),
            Content::Text(content) => Ok(serde_json::from_str(&content)?),
        }
    }
}

impl ResponseHandler for GenericResponseHandler {
    fn process(response: Response) -> Result<ResponseHandlerResult, Error> {
        match response.status().as_u16() {
            200 => Ok(ResponseHandlerResult::Ok(Content::Response(response))),

            _ => Ok(ResponseHandlerResult::Retry(response.status().as_u16().to_string())),
        }
    }
}

impl ResponseHandler for EtherscanResponseHandler {
    fn process(response: Response) -> Result<ResponseHandlerResult, Error> {
        #[derive(Deserialize)]
        struct Page {
            status: String,
            result: String,
        }

        match response.status().as_u16() {
            200 => {
                let url = response.url().to_string();
                let content = response.text().unwrap();
                let json = serde_json::from_str::<Page>(&content)?;

                // This is such a stupid fucking convention but Etherscan (among others) always return a 200
                // status code regardless of whether or not the request was sucessful. The actual status
                // is wrapped within a JSON body that is returned when receiving a 200 status code.
                // We therefore have to parse the JSON to handle the response.
                match json.status.as_str() {
                    "1" => Ok(ResponseHandlerResult::Ok(Content::Text(content))),

                    // Anything other than a "1" as a JSON status is an error
                    _ => match json.result.as_str() {
                        "Invalid API Key" => Err(Error::EtherscanInvalidToken(url)),

                        "Contract source code not verified" => {
                            Err(Error::EtherscanContractSourceCodeNotVerified(url))
                        }

                        "Max rate limit reached" => {
                            // 5 API calls per seconds, hence sleep 1 seconds before retrying
                            Ok(ResponseHandlerResult::RetryWithCustomSleepDuration(1))
                        }

                        _ => Ok(ResponseHandlerResult::Retry(json.result)),
                    },
                }
            }

            _ => Ok(ResponseHandlerResult::Retry(response.status().as_u16().to_string())),
        }
    }
}

impl ResponseHandler for GithubResponseHandler {
    fn prepare(request_handler: &RequestHandler, url: &str) -> RequestBuilder {
        let mut request = request_handler.client.get(url);
        request = request.header(header::USER_AGENT, GITHUB_USER_AGENT);
        request = request.bearer_auth(&request_handler.github_tokenmanager.as_ref().unwrap().borrow().active);
        request = request.query(&[("per_page", "100")]);

        request
    }

    fn process(response: Response) -> Result<ResponseHandlerResult, Error> {
        match response.status().as_u16() {
            200 => Ok(ResponseHandlerResult::Ok(Content::Response(response))),

            // The 304 status code is returned only when using conditional request[0] as such the response
            // itself is no error. We therefore return a Ok(response) which the `modified_since` method within
            // the `RepoHandler` module can use to determine whether a resource was modified or not.
            // [0] https://docs.github.com/en/rest/overview/resources-in-the-rest-api#conditional-requests
            304 => Ok(ResponseHandlerResult::Ok(Content::Response(response))),

            // The currently used token is invalid because it e.g. expired, hence clean up the token pool
            // before retrying.
            401 => Ok(ResponseHandlerResult::RetryWithAction(Action::GithubCleanup)),

            // GitHub returns a 403 error either because:
            // - The requested resource is unavailable in which case we return an error or because
            // - The currently used token has reached its ratelimit in which case we replace the token with
            //   another one in the token pool before retrying.
            403 => {
                let url = response.url().to_string();

                match github_parse_error_message(response).contains("access blocked") {
                    true => Err(Error::GithubResourceUnavailable(url)),
                    false => Ok(ResponseHandlerResult::RetryWithAction(Action::GithubRefresh)),
                }
            }

            404 | 451 => Err(Error::GithubResourceUnavailable(response.url().to_string())),

            _ => Ok(ResponseHandlerResult::Retry(response.status().as_u16().to_string())),
        }
    }
}

impl ResponseHandler for TokenManagerResponseHandler {
    fn prepare(request_handler: &RequestHandler, url: &str) -> RequestBuilder {
        let mut request = request_handler.client.get(url);
        request = request.header(header::USER_AGENT, GITHUB_USER_AGENT);

        request
    }

    fn process(response: Response) -> Result<ResponseHandlerResult, Error> {
        match response.status().as_u16() {
            200 => Ok(ResponseHandlerResult::Ok(Content::Response(response))),

            // Just like in the GithubResponseHandler, a 401 status code is used when the currently used token
            // is invalid, e.g. because it expired.
            401 => Err(Error::GithubTokenInvalid),

            _ => Ok(ResponseHandlerResult::Retry(response.status().as_u16().to_string())),
        }
    }
}

fn github_parse_error_message(response: Response) -> String {
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
