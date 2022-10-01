//! Function calls in the Ethereum network are specified by the first four byte of data sent with a transaction.
//! These first four bytes, also called function selector, represent the Keccak256 hash of the functions canonical form (e.g. `balanceOf(address)`).
//! As an outsider it is impossible to interpret what a given transaction does, because hashes are one-way
//! calculations. [Events](https://medium.com/mycrypto/understanding-event-logs-on-the-ethereum-blockchain-f4ae7ba50378)
//! and [errors](https://blog.soliditylang.org/2021/04/21/custom-errors/) are encoded in a similar fashion. As such rainbow tables are
//! needed to decode and inspect such signatures in the Ethereum network. While such rainbow tables exists,
//! most prominently [4Byte](https://www.4byte.directory/), two features are missing which Etherface tries to cover.
//! First, finding such signatures automatically from various websites where such signatures can be found
//! (currently GitHub, Etherscan and 4Byte) without any human intervention whatsoever. Second, providing source code references
//! where these signatures were found. For comparision, 4Byte relies on user submitted data / GitHub Webhooks
//! for the former and does not support the latter at all.
//!
//! The architecture for Etherface looks as follows
//! <div align="center">
//!  <img src="https://github.com/volsa/etherface/blob/master/res/img/architecture_etherface.png?raw=true">
//! </div>
//!
//! The `fetcher` module is thereby responsible for finding [Solidity](https://docs.soliditylang.org/en/latest/)
//! files where such signatures are present by either crawling or polling websites whereas the `scraper` module
//! is responsible for downloading these files, scraping all function, event and error signatures inserting
//! them into the database. These scraped signatures are then publicly available at <https://etherface.io/>.

mod fetcher;
mod scraper;

extern crate log;
extern crate simplelog;

use crate::fetcher::etherscan::EtherscanFetcher;
use crate::fetcher::fourbyte::FourbyteFetcher;
use crate::fetcher::Fetcher;
use crate::scraper::etherscan::EtherscanScraper;
use crate::scraper::github::GithubScraper;
use crate::scraper::Scraper;
use anyhow::Error;
use fetcher::github::GithubFetcher;
use log::debug;
use simplelog::CombinedLogger;
use simplelog::*;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

fn main() -> Result<(), Error> {
    CombinedLogger::init(vec![
        TermLogger::new(
            // LevelFilter::max(),
            LevelFilter::Debug,
            ConfigBuilder::new()
                .add_filter_allow_str("etherface")
                .set_time_format_str("[%d.%m.%Y; %T]")
                .build(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            ConfigBuilder::new()
                .add_filter_allow_str("etherface")
                .set_time_format_str("[%d.%m.%Y; %T]")
                .build(),
            std::fs::OpenOptions::new().append(true).create(true).open("etherface.log")?,
        ),
    ])
    .unwrap();

    let (tx, rx) = mpsc::channel();
    start_data_retrieval_threads(&tx);
    start_data_scraper_threads(&tx);

    // This block until we receive a message, which in turn we only receive if there was an error
    match rx.recv() {
        Ok(msg) => anyhow::bail!(msg),
        Err(why) => anyhow::bail!(why),
    }
}

fn start_data_scraper_threads(tx: &Sender<Error>) {
    let scrapers: Vec<Box<dyn Scraper + Sync + Send>> =
        vec![Box::new(GithubScraper), Box::new(EtherscanScraper)];

    for scraper in scrapers {
        let tx_abort_channel = tx.clone();

        std::thread::spawn(move || {
            debug!("Starting scraper {:#?}", scraper);

            if let Err(why) = scraper.start() {
                tx_abort_channel.send(why).unwrap();
            }
        });
    }
}

fn start_data_retrieval_threads(tx: &Sender<Error>) {
    let fetchers: Vec<Box<dyn Fetcher + Sync + Send>> = vec![
        Box::new(FourbyteFetcher),
        Box::new(EtherscanFetcher),
        Box::new(GithubFetcher),
    ];

    for fetcher in fetchers {
        let tx_abort_channel = tx.clone();

        std::thread::spawn(move || {
            debug!("Starting fetcher {:#?}", fetcher);

            if let Err(why) = fetcher.start() {
                tx_abort_channel.send(why).unwrap();
            }
        });
    }
}
