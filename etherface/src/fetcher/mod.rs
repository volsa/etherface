//! Consists of sub-modules responsible for finding Solidity files from various websites.

pub mod etherscan;
pub mod fourbyte;
pub mod github;

use anyhow::Error;

/// Sleep duration between fetching iterations; used only for fetchers where polling is present, i.e.
/// [`etherscan`] and [`fourbyte`].
const FETCHER_POLLING_SLEEP_TIME: u64 = 5 * 60;

/// Trait providing the entry point for starting a fetcher.
pub trait Fetcher: std::fmt::Debug {
    /// Starts the fetching process.
    fn start(&self) -> Result<(), Error>;
}
