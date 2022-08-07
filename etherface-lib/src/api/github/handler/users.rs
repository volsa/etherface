use crate::api::github::page::Page;
use crate::api::github::GithubClient;
use crate::error::Error;
use crate::model::GithubRepository;
use crate::model::GithubUser;

pub struct UserHandler<'a> {
    ghc: &'a GithubClient,
    id: i32,
}

impl<'a> UserHandler<'a> {
    pub fn new(ghc: &'a GithubClient, id: i32) -> Self {
        UserHandler { ghc, id }
    }

    pub fn get(&self) -> Result<GithubUser, Error> {
        let path = format!("user/{id}", id = self.id);
        Ok(self.ghc.execute(&path)?.json().unwrap())
    }

    pub fn starred(&self) -> Result<Vec<GithubRepository>, Error> {
        let path = format!("user/{id}/starred", id = self.id,);
        Page::all_pages(self.ghc, path)
    }

    pub fn repos(&self) -> Result<Vec<GithubRepository>, Error> {
        let path = format!("user/{id}/repos", id = self.id,);
        Page::all_pages(self.ghc, path)
    }
}

#[cfg(test)]
mod tests {
    use crate::api::github::GithubClient;

    #[test]
    fn get() {
        let ghc = GithubClient::new().unwrap();

        let user = ghc.user(29666622).get().unwrap();
        assert_eq!(user.login, "volsa");
        assert_eq!(user.public_repos, Some(3));
    }

    #[test]
    fn starred() {
        let ghc = GithubClient::new().unwrap();

        let starred = ghc.user(29666622).starred().unwrap();
        assert_eq!(starred.len(), 6);

        let starred_names: Vec<String> = starred.into_iter().map(|x| x.name).collect();
        assert!(starred_names.contains(&"EIPs".to_string()));
        assert!(starred_names.contains(&"hashbrown".to_string()));
    }

    #[test]
    fn repos() {
        let ghc = GithubClient::new().unwrap();

        let repos = ghc.user(29666622).repos().unwrap();
        assert_eq!(repos.len(), 3);

        let repo_names: Vec<String> = repos.into_iter().map(|x| x.name).collect();
        assert!(repo_names.contains(&"http-cpp".to_string())); // TODO:
    }
}
