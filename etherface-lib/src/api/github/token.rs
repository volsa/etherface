use crate::api::github::GITHUB_RATELIMIT_URL;
use crate::api::RequestHandler;
use crate::api::TokenManagerResponseHandler;
use crate::config::Config;
use crate::error::Error;
use log::info;
use log::warn;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RatelimitRoot {
    pub resources: RatelimitObject,
}

#[derive(Debug, Deserialize)]
struct RatelimitObject {
    pub core: Ratelimit,
    pub search: Ratelimit,
}

#[derive(Debug, Deserialize)]
struct Ratelimit {
    pub remaining: usize,
}

pub(crate) struct TokenManager {
    pub active: String,
    pool: Vec<String>,
    request_handler: Box<RequestHandler>,
}

impl TokenManager {
    pub fn new() -> Result<Self, Error> {
        let tokens = Config::new()?.github.tokens;

        let mut manager = TokenManager {
            active: tokens[0].clone(),
            pool: tokens,
            request_handler: Box::new(RequestHandler::new()),
        };
        manager.cleanup()?; // Make sure we have only valid tokens before returning the TokenManager

        Ok(manager)
    }

    /// Find and replace the active GitHub token with one that has more API calls.
    /// If none can be found, that is all tokens are drained, this method will sleep for 5 minutes.
    pub fn refresh(&mut self) -> Result<(), Error> {
        if let Ok(ratelimit) = self.execute(&self.active) {
            if ratelimit.search.remaining == 0 {
                // The search ratelimit resets every minute, as such we can sleep for one minute
                // instead of hotswapping the active token. This makes the method much more readable
                // and has less of an overhead.
                // See the docs for the differences between the core and search ratelimit:
                // https://docs.github.com/en/rest/overview/resources-in-the-rest-api#rate-limiting
                // https://docs.github.com/en/rest/reference/search#rate-limit
                info!("Search ratelimit drained, sleeping one minute to reset");
                std::thread::sleep(std::time::Duration::from_secs(60));
                return Ok(());
            }
        }

        let mut valid_tokens: Vec<(&str, usize)> = Vec::new();
        for token in &self.pool {
            if let Ok(ratelimit) = self.execute(token) {
                valid_tokens.push((token, ratelimit.core.remaining));
            }
        }

        if valid_tokens.is_empty() {
            return Err(Error::GithubTokenPoolEmpty);
        }

        let mut best = &valid_tokens[0];
        for token in &valid_tokens {
            if token.1 > best.1 {
                best = token;
            }
        }

        match best.1 {
            0 => {
                let sleep_duration = 5 * 60;
                info!("All tokens drained, sleeping {} seconds", sleep_duration);
                std::thread::sleep(std::time::Duration::from_secs(sleep_duration));
            }
            _ => {
                info!("Replacing activen token {} with {}", self.active, best.0);
                self.active = best.0.to_string();
            }
        }

        Ok(())
    }

    /// Find and remove all invalid tokens from the token pool.
    pub fn cleanup(&mut self) -> Result<(), Error> {
        let mut invalid_tokens: Vec<String> = Vec::new();
        for token in &self.pool {
            // if let Err(Error::GithubTokenInvalid(token)) = self.execute(token) {
            if let Err(Error::GithubTokenInvalid) = self.execute(token) {
                invalid_tokens.push(token.to_string());
            }
        }

        if invalid_tokens.len() == self.pool.len() {
            return Err(Error::GithubTokenPoolEmpty);
        }

        for token in invalid_tokens {
            warn!("Removing expired / invalid token {}", token);
            self.pool.retain(|x| *x != token);
        }

        // Replace the activen token in case it _might_ have been removed from the pool
        info!("Replacing active token {} with {}", self.active, self.pool[0]);
        self.active = self.pool[0].to_string();
        Ok(())
    }

    fn execute(&self, token: &str) -> Result<RatelimitObject, Error> {
        Ok(self
            .request_handler
            .execute_deser_token::<TokenManagerResponseHandler, RatelimitRoot>(GITHUB_RATELIMIT_URL, token)?
            .resources)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::github::token::TokenManager;
    use crate::error::Error;
    use reqwest::blocking::Client;
    use reqwest::StatusCode;

    const INVALID_TOKEN_0: &str = "ghp_INVALIDuMzJHt21404WDydRCjB7PINVALID0";
    const INVALID_TOKEN_1: &str = "ghp_INVALIDuMzJHt21404WDydRCjB7PINVALID1";
    const INVALID_TOKEN_2: &str = "ghp_INVALIDuMzJHt21404WDydRCjB7PINVALID2";

    #[test]
    fn refresh() {
        let mut token_manager = TokenManager::new().unwrap();
        assert!(token_manager.pool.len() >= 3, "Need at least 3 valid tokens");
        token_manager.pool.truncate(3); // 3 tokens are more than plenty for this test

        let mut token_ratelimit_tuple: Vec<(String, usize)> = Vec::new();
        for token in &token_manager.pool {
            let remaining = token_manager.execute(token).unwrap().core.remaining;
            token_ratelimit_tuple.push((token.to_string(), remaining));
        }

        // This test works as follows:
        // 1. Sort the tokens by their ratelimit where the first token in the __sorted__ vector
        //    is the token with the least remaining API calls.
        // 2. Drain the ratelimit of the first two tokens by a small amount. This is only needed
        //    in cases where all tokens have their max remainig API calls and as such refresh()
        //    would always pick the first token since they're all the same).
        // 3. Test refresh() by assigning:
        //      - the token with the least remaining API calls to be the active token
        //      - the token with the second least remaining API calls to be the active token
        //      - the token with the most remaining API calls to be the active token
        //    In all those cases the token with the most remaining API calls should be the active
        //    token after refresh() has been called.
        token_ratelimit_tuple.sort_by(|a, b| a.1.cmp(&b.1));

        drain_token(&token_ratelimit_tuple[0].0, 4);
        drain_token(&token_ratelimit_tuple[1].0, 2);

        token_manager.active = token_ratelimit_tuple[0].0.clone();
        token_manager.refresh().unwrap();
        assert_eq!(token_manager.active, token_ratelimit_tuple[2].0);

        token_manager.active = token_ratelimit_tuple[1].0.clone();
        token_manager.refresh().unwrap();
        assert_eq!(token_manager.active, token_ratelimit_tuple[2].0);

        token_manager.active = token_ratelimit_tuple[2].0.clone();
        token_manager.refresh().unwrap();
        assert_eq!(token_manager.active, token_ratelimit_tuple[2].0);
    }

    #[test]
    fn cleanup_every_token_valid() {
        let mut token_manager = TokenManager::new().unwrap();
        assert!(token_manager.pool.len() >= 3, "Need at least 3 valid tokens");

        // Check if all tokens are valid
        for token in &token_manager.pool {
            assert!(get_status_code_on_ratelimit_endpoint(token).is_success());
        }

        // Check if the token pool remains unchanged if cleanup() is called on a valid token pool
        let prev_pool_size = token_manager.pool.len();
        token_manager.cleanup().unwrap();
        assert_eq!(token_manager.pool.len(), prev_pool_size);
    }

    #[test]
    fn cleanup_every_token_valid_but_one() {
        let mut token_manager = TokenManager::new().unwrap();
        assert!(token_manager.pool.len() >= 3, "Need at least 3 valid tokens");

        // Check if all tokens are valid
        for token in &token_manager.pool {
            assert!(get_status_code_on_ratelimit_endpoint(token).is_success());
        }

        token_manager.pool[0] = INVALID_TOKEN_0.to_string();

        // Check if the token pool shrinks by one
        let prev_pool_size = token_manager.pool.len();
        token_manager.cleanup().unwrap();
        assert_eq!(token_manager.pool.len(), prev_pool_size - 1);
    }

    #[test]
    fn cleanup_every_token_invalid() {
        let mut token_manager = TokenManager::new().unwrap();

        token_manager.pool.clear();
        token_manager.pool.push(INVALID_TOKEN_0.to_string());
        token_manager.pool.push(INVALID_TOKEN_1.to_string());
        token_manager.pool.push(INVALID_TOKEN_2.to_string());

        assert!(token_manager.cleanup().is_err());
        assert_eq!(
            // Kind of ugly but eeeh at least it works without implementing the PartialEq trait
            token_manager.cleanup().unwrap_err().to_string(),
            Error::GithubTokenPoolEmpty.to_string()
        );
    }

    #[test]
    fn cleanup_every_token_invalid_but_one() {
        let mut token_manager = TokenManager::new().unwrap();
        assert!(token_manager.pool.len() >= 3, "Need at least 3 valid tokens");

        // Check if all tokens are valid
        for token in &token_manager.pool {
            assert!(get_status_code_on_ratelimit_endpoint(token).is_success());
        }

        // Remove all tokens but one then insert invalid tokens
        token_manager.pool.truncate(1);
        token_manager.pool.push(INVALID_TOKEN_0.to_string());
        token_manager.pool.push(INVALID_TOKEN_1.to_string());
        token_manager.pool.push(INVALID_TOKEN_2.to_string());

        token_manager.cleanup().unwrap();
        assert_eq!(token_manager.pool.len(), 1);
    }

    fn get_status_code_on_ratelimit_endpoint(token: &str) -> StatusCode {
        let http_client = Client::builder().user_agent("Etherface").build().unwrap();
        http_client.get("https://api.github.com/rate_limit").bearer_auth(token).send().unwrap().status()
    }

    fn drain_token(token: &str, amount: usize) {
        let http_client = Client::builder().user_agent("Etherface").build().unwrap();

        for _ in 0..amount {
            http_client.get("https://api.github.com/users/ethereum").bearer_auth(token).send().unwrap();
        }
    }
}
