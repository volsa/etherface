use crate::api::github::page::Page;
use crate::api::github::GithubClient;
use crate::error::Error;
use crate::model::GithubRepository;
use crate::model::GithubUser;
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;

pub struct RepoHandler<'a> {
    ghc: &'a GithubClient,
    id: i32,
}

impl<'a> RepoHandler<'a> {
    pub fn new(ghc: &'a GithubClient, id: i32) -> Self {
        RepoHandler { ghc, id }
    }

    pub fn get(&self) -> Result<GithubRepository, Error> {
        let path = format!("repositories/{id}", id = self.id);

        Ok(self.ghc.execute(&path)?.json().unwrap())
    }

    pub fn stargazers(&self) -> Result<Vec<GithubUser>, Error> {
        let path = format!("repositories/{id}/stargazers", id = self.id);

        Page::all_pages(self.ghc, path)
    }

    pub fn languages(&self) -> Result<HashMap<String, usize>, Error> {
        let path = format!("repositories/{id}/languages", id = self.id);

        Ok(self.ghc.execute(&path)?.json().unwrap())
    }

    pub fn forks(&self) -> Result<Vec<GithubRepository>, Error> {
        let path = format!("repositories/{id}/forks", id = self.id);

        Page::all_pages(self.ghc, path)
    }

    pub fn solidity_ratio(&self) -> Result<f32, Error> {
        let languages = self.languages()?;
        let solidity = languages.get("Solidity");

        if languages.is_empty() || solidity.is_none() {
            return Ok(0.0);
        }

        let total: usize = languages.iter().map(|x| x.1).sum();
        Ok(*solidity.unwrap() as f32 / total as f32)
    }

    // https://docs.github.com/en/rest/overview/resources-in-the-rest-api#conditional-requests
    pub fn modified_since(&self, date: DateTime<Utc>) -> Result<Option<GithubRepository>, Error> {
        let path = format!("repositories/{id}", id = self.id);
        let date = date.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        let kv = ("If-Modified-Since", date.as_str());

        let response = self.ghc.execute_with_header(&path, kv)?;

        match response.status().as_u16() == 304 {
            true => Ok(None),
            false => Ok(Some(response.json()?)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::github::GithubClient;
    use chrono::TimeZone;
    use chrono::Utc;

    #[test]
    fn get() {
        let ghc = GithubClient::new().unwrap();
        let repo = ghc.repos(44971752).get().unwrap();
        assert_eq!(repo.id, 44971752);
        assert_eq!(repo.name, "EIPs");
        assert_eq!(repo.html_url, "https://github.com/ethereum/EIPs");
        assert_eq!(repo.language, Some("Solidity".to_string()));
    }

    #[test]
    fn stargazers() {
        let ghc = GithubClient::new().unwrap();

        let http_client = reqwest::blocking::Client::default();
        let response = http_client.get("https://github.com/ethereum/EIPs").send().unwrap();
        let html_content = response.text().unwrap();

        // The HTML content should contain something along: 'aria-label="[...] users starred this repository"'
        let stargazers = ghc.repos(44971752).stargazers().unwrap();
        assert!(html_content.contains(&format!("{} users starred this repository", stargazers.len())));

        let stargazer_names: Vec<String> = stargazers.into_iter().map(|x| x.login).collect();
        assert!(stargazer_names.contains(&"volsa".to_string()));
    }

    #[test]
    fn languages() {
        let ghc = GithubClient::new().unwrap();
        let languages = ghc.repos(44971752).languages().unwrap();

        assert_eq!(languages.len(), 8);
        assert_eq!(languages.contains_key("Solidity"), true);
        assert_eq!(languages.contains_key("JavaScript"), true);
        assert_eq!(languages.contains_key("TypeScript"), true);
        assert_eq!(languages.contains_key("HTML"), true);
        assert_eq!(languages.contains_key("C++"), true);
        assert_eq!(languages.contains_key("Python"), true);
        assert_eq!(languages.contains_key("SCSS"), true);
        assert_eq!(languages.contains_key("Ruby"), true);
    }

    #[test]
    fn fork() {
        let ghc = GithubClient::new().unwrap();
        let sushiswap_repo_parent = ghc.repos(44971752).get().unwrap();
        let sushiswap_repo_fork = ghc.repos(81120900).get().unwrap();

        assert_eq!(sushiswap_repo_fork.fork, true);
        assert_eq!(sushiswap_repo_fork.fork_parent.unwrap().id, sushiswap_repo_parent.id);
    }

    #[test]
    fn forks() {
        let ghc = GithubClient::new().unwrap();

        let http_client = reqwest::blocking::Client::default();
        let response = http_client.get("https://github.com/ethereum/EIPs").send().unwrap();
        let html_content = response.text().unwrap();

        // The HTML content should contain something along: 'aria-label="[...] users starred this repository"'
        let stargazers = ghc.repos(44971752).stargazers().unwrap();
        assert!(html_content.contains(&format!("{} users starred this repository", stargazers.len())));

        let stargazer_names: Vec<String> = stargazers.into_iter().map(|x| x.login).collect();
        assert!(stargazer_names.contains(&"volsa".to_string()));
    }

    #[test]
    fn solidity_ratio() {
        let ghc = GithubClient::new().unwrap();
        let ratio = ghc.repos(44971752).solidity_ratio().unwrap();

        // https://api.github.com/repos/ethereum/EIPs/languages
        assert!(ratio >= 0.6 && ratio <= 0.65);
    }

    #[test]
    fn where_modified_since() {
        let ghc = GithubClient::new().unwrap();
        let repo = ghc.repos(44971752).get().unwrap();

        assert_eq!(
            Some(repo),
            ghc.repos(44971752).modified_since(Utc.ymd(2022, 01, 01).and_hms(0, 0, 0)).unwrap()
        );
    }

    #[test]
    fn where_not_modified_since() {
        let ghc = GithubClient::new().unwrap();

        assert_eq!(None, ghc.repos(44971752).modified_since(Utc::now()).unwrap());
    }
}
