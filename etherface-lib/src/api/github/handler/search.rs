//! `/search` endpoint handler.

use crate::api::github::page::Page;
use crate::api::github::GithubClient;
use crate::error::Error;
use crate::model::GithubRepository;
use chrono::Date;
use chrono::Utc;

pub struct SearchHandler<'a> {
    ghc: &'a GithubClient,
}

impl<'a> SearchHandler<'a> {
    pub(crate) fn new(ghc: &'a GithubClient) -> Self {
        SearchHandler { ghc }
    }

    /// Returns the deserialized JSON `/search/repositories?q={query}` response.
    pub fn repos(&self, query: &str) -> Result<Vec<GithubRepository>, Error> {
        let path = format!("search/repositories?q={query}");
        Page::all_pages(self.ghc, path)
    }

    /// Returns the deserialized JSON `/search/repositories?q=language:solidity created:{date}` response.
    pub fn solidity_repos_created_at(&self, date: Date<Utc>) -> Result<Vec<GithubRepository>, Error> {
        self.repos(&format!("language:solidity created:{}", date.format("%Y-%m-%d")))
    }

    /// Returns the deserialized JSON `/search/repositories?q=language:solidity pushed:{date}` response.
    pub fn solidity_repos_updated_at(&self, date: Date<Utc>) -> Result<Vec<GithubRepository>, Error> {
        self.repos(&format!("language:solidity pushed:{}", date.format("%Y-%m-%d")))
    }
}

#[cfg(test)]
mod tests {
    use crate::api::github::GithubClient;
    use chrono::TimeZone;
    use chrono::Utc;

    #[test]
    fn repos() {
        let ghc = GithubClient::new().unwrap();

        let search = ghc.search().repos("language:Solidity").unwrap();
        assert_eq!(search.len(), 1000);

        let search_names: Vec<String> = search.into_iter().map(|x| x.name).collect();
        assert!(search_names.contains(&"EIPs".to_string()));
        assert!(search_names.contains(&"solidity-examples".to_string()));
    }

    #[test]
    fn solidity_repos_created_at() {
        let ghc = GithubClient::new().unwrap();

        // https://api.github.com/search/repositories?q=language:solidity%20created:2022-01-01&per_page=100
        let search = ghc.search().solidity_repos_created_at(Utc.ymd(2022, 1, 1)).unwrap();
        assert_eq!(search.len(), 95);
    }

    #[test]
    fn solidity_repos_updated_at() {
        let ghc = GithubClient::new().unwrap();

        // https://api.github.com/search/repositories?q=language:solidity%20pushed:2022-01-01&per_page=100
        let search = ghc.search().solidity_repos_updated_at(Utc.ymd(2022, 1, 1)).unwrap();
        assert_eq!(search.len(), 81);
    }
}
