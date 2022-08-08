pub mod etherscan;
pub mod github;

use anyhow::Error;

/// Sleep duration between scraping iterations 
const SCRAPER_SLEEP_DURATION: u64 = 5 * 60;

/// Trait providing the entry point for starting a scraper.
pub trait Scraper: std::fmt::Debug {
    /// Starts the scraping process.
    fn start(&self) -> Result<(), Error>;
}
