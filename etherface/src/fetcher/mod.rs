pub mod etherscan;
pub mod fourbyte;
pub mod github;

use anyhow::Error;

const FETCHER_LOOP_SLEEP_TIME: u64 = 5 * 60;

pub trait Fetcher: std::fmt::Debug {
    fn start(&self) -> Result<(), Error>;
}
