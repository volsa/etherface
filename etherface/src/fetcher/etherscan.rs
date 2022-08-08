//! Fetcher for <https://etherscan.io/>
//! 
//! Polls the <https://etherscan.io/contractsVerified> site every [`FETCHER_POLLING_SLEEP_TIME`], extracting
//! all contract metadata inserting them into the database (if not already present). 
use crate::fetcher::Fetcher;
use crate::fetcher::FETCHER_POLLING_SLEEP_TIME;
use anyhow::Error;
use etherface_lib::api::etherscan::EtherscanClient;
use etherface_lib::database::handler::DatabaseClient;

#[derive(Debug)]
pub struct EtherscanFetcher;

impl Fetcher for EtherscanFetcher {
    fn start(&self) -> Result<(), Error> {
        let esc = EtherscanClient::new()?;
        let dbc = DatabaseClient::new()?;

        loop {
            for contract in esc.get_verified_contracts()? {
                dbc.etherscan_contract().insert(&contract);
            }

            std::thread::sleep(std::time::Duration::from_secs(FETCHER_POLLING_SLEEP_TIME));
        }
    }
}
