use crate::fetcher::etherscan::EtherscanFetcher;
use crate::fetcher::fourbyte::FourbyteFetcher;
use crate::fetcher::Fetcher;
use crate::scraper::etherscan::EtherscanScraper;
use crate::scraper::github::GithubScraper;
use crate::scraper::Scraper;
use anyhow::Error;
use fetcher::github::GithubFetcher;
use simplelog::CombinedLogger;
use simplelog::*;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

mod fetcher;
mod scraper;
extern crate log;
extern crate simplelog;

enum ThreadStatus {
    Abort(String),
}

fn main() -> Result<(), Error> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::max(),
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
            std::fs::OpenOptions::new().append(true).open("etherface.log")?,
        ),
    ])
    .unwrap();

    let (tx, rx) = mpsc::channel();
    start_data_retrieval_threads(&tx);
    start_data_scraper_threads(&tx);

    // This block until we receive a message, which in turn we only receive if there was an error
    match rx.recv() {
        Ok(msg) => match msg {
            ThreadStatus::Abort(why) => anyhow::bail!("{why}"),
        },

        Err(why) => anyhow::bail!("{why}"),
    }
}

fn start_data_scraper_threads(tx: &Sender<ThreadStatus>) {
    let scrapers: Vec<Box<dyn Scraper + Sync + Send>> =
        vec![Box::new(GithubScraper), Box::new(EtherscanScraper)];

    for scraper in scrapers {
        let tx_abort_channel = tx.clone();

        std::thread::spawn(move || {
            println!("Starting scraper {:#?}", scraper);

            if let Err(why) = scraper.start() {
                tx_abort_channel.send(ThreadStatus::Abort(why.to_string())).unwrap();
            }
        });
    }
}

fn start_data_retrieval_threads(tx: &Sender<ThreadStatus>) {
    let fetchers: Vec<Box<dyn Fetcher + Sync + Send>> = vec![
        Box::new(FourbyteFetcher),
        Box::new(EtherscanFetcher),
        Box::new(GithubFetcher),
    ];

    for fetcher in fetchers {
        let tx_abort_channel = tx.clone();

        std::thread::spawn(move || {
            println!("Starting fetcher {:#?}", fetcher);

            if let Err(why) = fetcher.start() {
                tx_abort_channel.send(ThreadStatus::Abort(why.to_string())).unwrap();
            }
        });
    }
}
