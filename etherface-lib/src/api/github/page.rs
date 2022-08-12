//! Handles all GitHub API pagination logic. For a better understanding of this module make sure to read
//! <https://docs.github.com/en/rest/overview/resources-in-the-rest-api#pagination>,

use crate::api::github::GithubClient;
use crate::error::Error;
use hyperx::header::TypedHeaders;
use log::warn;
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;

pub(crate) struct Page<T> {
    items: Vec<T>,
    rel_next: Option<String>,
}

impl<T> Page<T>
where
    T: DeserializeOwned,
{
    pub fn all_pages(ghc: &GithubClient, path: String) -> Result<Vec<T>, Error> {
        let mut items = Vec::new();
        let mut page = get_page(ghc, &path)?;

        items.append(&mut page.items); // append items from first page before iterating
        while let Some(rel_next) = page.rel_next {
            page = get_page(ghc, &rel_next)?;
            items.append(&mut page.items);
        }

        Ok(items)
    }
}

fn get_page<T>(ghc: &GithubClient, url: &str) -> Result<Page<T>, Error>
where
    T: DeserializeOwned,
{
    let response = ghc.execute(url)?;
    let rel_next = get_rel_next(response.headers());

    let json_response = match response.json() {
        Ok(val) => val,
        Err(why) => {
            warn!("Failed to parse JSON on page {url}; {why}");
            return Ok(Page {
                rel_next,
                items: Vec::with_capacity(0),
            });
        }
    };

    let json: serde_json::Value = serde_json::from_value(json_response).unwrap();

    // Sometimes the actual elements are wrapped inside an "items" JSON array, as such we have to
    // first extract those items before deserialzing them. To better illustrate it, compare the
    // contents of the following two links
    // - https://api.github.com/search/repositories?q=ethereum (wrapped inside "items" array)
    // - https://api.github.com/users/ethereum/repos (not wrapped, response is a JSON array)
    let items = match json.is_array() {
        true => serde_json::from_value(json),
        false => serde_json::from_value(json.get("items").cloned().unwrap()),
    };

    match items {
        Ok(val) => Ok(Page { items: val, rel_next }),

        Err(why) => {
            warn!("Failed to parse page {}; {}", url, why);
            Ok(Page {
                rel_next,

                // Some Pages contain a '"owner": null"' field which indicates that the repository owner no longer
                // is available (deleted, banned, etc..). However such cases are super rare hence the owner field
                // in `model::Repository` is not an Option because it wouldn't make sense to add checks
                // (if let Some(...)) for such rare cases therefore we return an empty vector instead.
                // For reference this page (may no longer be the case) contains such a null owner field:
                // https://api.github.com/user/16433547/starred?per_page=100&page=61
                items: Vec::new(),
            })
        }
    }
}

fn get_rel_next(headers: &HeaderMap) -> Option<String> {
    let mut rel_next = None;

    if let Ok(link_header) = headers.decode::<hyperx::header::Link>() {
        for value in link_header.values() {
            if let Some(&[hyperx::header::RelationType::Next]) = value.rel() {
                rel_next = Some(value.link().to_string());
            }
        }
    }

    rel_next
}
