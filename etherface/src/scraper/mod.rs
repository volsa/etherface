pub mod etherscan;
pub mod github;

use anyhow::Error;

pub trait Scraper: std::fmt::Debug {
    fn start(&self) -> Result<(), Error>;
}
