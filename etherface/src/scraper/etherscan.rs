//! Scraper for <https://etherscan.io/>
//!
//! Fetches all unscraped Etherscan contract addresses from the database, downloads their ABI content using
//! the <https://api.etherscan.io/api?module=contract&action=getabi> endpoint extracting signatures. These
//! extracted signatures are then inserted into the database with a reference to the contract address, marking
//! the contract as scraped. The whole process is then repeated every [`SCRAPER_SLEEP_DURATION`] seconds.

use crate::scraper::Scraper;
use anyhow::Error;
use chrono::Utc;
use etherface_lib::api::etherscan::EtherscanClient;
use etherface_lib::database::handler::DatabaseClient;
use etherface_lib::model::MappingSignatureEtherscan;
use etherface_lib::parser;

use super::SCRAPER_SLEEP_DURATION;

#[derive(Debug)]
pub struct EtherscanScraper;
impl Scraper for EtherscanScraper {
    fn start(&self) -> Result<(), Error> {
        let dbc = DatabaseClient::new()?;
        let esc = EtherscanClient::new()?;

        loop {
            // Scrape signatures from unvisited contracts
            for contract in dbc.etherscan_contract().get_unvisited() {
                if let Ok(abi_content) = esc.get_abi(&contract.address) {
                    if let Ok(signatures) = parser::from_abi(&abi_content) {
                        // Insert all scraped signatures
                        for signature in signatures {
                            let inserted_signature = dbc.signature().insert(&signature);

                            let mapping = MappingSignatureEtherscan {
                                signature_id: inserted_signature.id,
                                contract_id: contract.id,
                                kind: signature.kind,
                                added_at: Utc::now(),
                            };

                            dbc.mapping_signature_etherscan().insert(&mapping);
                        }
                    }

                    dbc.etherscan_contract().set_visited(&contract);
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(SCRAPER_SLEEP_DURATION));
        }
    }
}
