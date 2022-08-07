use crate::config::Config;
use crate::error::Error;
use crate::model::EtherscanContract;
use chrono::Utc;
use select::document::Document;
use select::predicate::Name;
use select::predicate::Predicate;
use serde::Deserialize;

use super::EtherscanResponseHandler;
use super::GenericResponseHandler;
use super::RequestHandler;

pub struct EtherscanClient {
    request_handler: RequestHandler,
    token: String,
}

#[derive(Deserialize)]
struct Page {
    result: String,
}

impl EtherscanClient {
    pub fn new() -> Result<Self, Error> {
        let config = Config::new()?.etherscan;

        Ok(EtherscanClient {
            request_handler: RequestHandler::new(),
            token: config.token,
        })
    }

    pub fn get_abi(&self, address: &str) -> Result<String, Error> {
        let url = format!(
            "https://api.etherscan.io/api?module=contract&action=getabi&address={}&apikey={}",
            address, self.token
        );

        Ok(self.request_handler.execute_deser::<EtherscanResponseHandler, Page>(&url)?.result)
    }

    pub fn get_verified_contracts(&self) -> Result<Vec<EtherscanContract>, Error> {
        let mut contracts = Vec::new();

        // Each page can list a total of 100 contracts, thus iterate over 5 pages
        for idx in 1..=5 {
            let url = format!("https://etherscan.io/contractsVerified/{idx}?ps=100");
            let response = self.request_handler.execute_resp::<GenericResponseHandler>(&url)?;
            let document = Document::from(response.text().unwrap().as_ref());

            // Pick each row from https://etherscan.io/contractsVerified/ and extract their metadata
            for row in document.find(Name("tbody").child(Name("tr"))) {
                let row_column: Vec<String> = row.find(Name("td")).into_iter().map(|x| x.text()).collect();

                contracts.push(EtherscanContract {
                    id: 0, // Can be 0 because the ID gets a value assigned by the database (SERIAL type)
                    address: row_column[0].trim().to_string(),
                    name: row_column[1].trim().to_string(),
                    compiler: row_column[2].trim().to_string(),
                    compiler_version: row_column[3].trim().to_string(),
                    url: format!("https://etherscan.io/address/{}", row_column[0].trim()).to_string(),
                    visited_at: None,
                    added_at: Utc::now(),
                });
            }
        }

        Ok(contracts)
    }
}

#[cfg(test)]
mod test {
    use crate::api::etherscan::EtherscanClient;

    #[test]
    fn get_abi() {
        assert_eq!(
            EtherscanClient::new().unwrap().get_abi("0x4a25e19e0765ef63d7196728ac3c3f3119199555").unwrap(),
            "[{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"Approval\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"address\",\"name\":\"previousOwner\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\"}],\"name\":\"OwnershipTransferred\",\"type\":\"event\"},{\"anonymous\":false,\"inputs\":[{\"indexed\":true,\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\"},{\"indexed\":true,\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\"},{\"indexed\":false,\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\"}],\"name\":\"Transfer\",\"type\":\"event\"},{\"inputs\":[],\"name\":\"BASE_RATE\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"FishTank\",\"outputs\":[{\"internalType\":\"contract IFishTank\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"START\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"}],\"name\":\"allowance\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"name\":\"allowedAddresses\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\"}],\"name\":\"approve\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\"}],\"name\":\"balanceOf\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"user\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\"}],\"name\":\"burn\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"claimReward\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"decimals\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"subtractedValue\",\"type\":\"uint256\"}],\"name\":\"decreaseAllowance\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"user\",\"type\":\"address\"}],\"name\":\"getTotalClaimable\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"reciever\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"giftAmount\",\"type\":\"uint256\"}],\"name\":\"giftTokens\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"spender\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"addedValue\",\"type\":\"uint256\"}],\"name\":\"increaseAllowance\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"name\":\"lastUpdate\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"name\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"owner\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"pauseReward\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"renounceOwnership\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\"}],\"name\":\"rewards\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_address\",\"type\":\"address\"},{\"internalType\":\"bool\",\"name\":\"_access\",\"type\":\"bool\"}],\"name\":\"setAllowedAddresses\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"fishTankAddress\",\"type\":\"address\"}],\"name\":\"setFishTank\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"symbol\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address[]\",\"name\":\"giftAddresses\",\"type\":\"address[]\"},{\"internalType\":\"uint256[]\",\"name\":\"giftAmount\",\"type\":\"uint256[]\"}],\"name\":\"tokenGiftList\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"totalSupply\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\"}],\"name\":\"transfer\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"sender\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"recipient\",\"type\":\"address\"},{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\"}],\"name\":\"transferFrom\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"newOwner\",\"type\":\"address\"}],\"name\":\"transferOwnership\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"from\",\"type\":\"address\"},{\"internalType\":\"address\",\"name\":\"to\",\"type\":\"address\"}],\"name\":\"updateReward\",\"outputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\"}]"
        );
    }

    #[test]
    #[rustfmt::skip]
    fn get_verified_contracts() {
        let contracts = EtherscanClient::new().unwrap().get_verified_contracts().unwrap();
        let http_client = reqwest::blocking::Client::default();

        let html_content_page01 = http_client.get(format!("https://etherscan.io/contractsVerified/1?ps=100")).send().unwrap().text().unwrap();
        let html_content_page02 = http_client.get(format!("https://etherscan.io/contractsVerified/2?ps=100")).send().unwrap().text().unwrap();
        let html_content_page03 = http_client.get(format!("https://etherscan.io/contractsVerified/3?ps=100")).send().unwrap().text().unwrap();
        let html_content_page04 = http_client.get(format!("https://etherscan.io/contractsVerified/4?ps=100")).send().unwrap().text().unwrap();
        let html_content_page05 = http_client.get(format!("https://etherscan.io/contractsVerified/5?ps=100")).send().unwrap().text().unwrap();

        assert!(html_content_page01.contains(&contracts[50].address));
        assert!(html_content_page01.contains(&contracts[50].name));
        assert!(html_content_page01.contains(&contracts[50].compiler));
        assert!(html_content_page01.contains(&contracts[50].compiler_version));
        
        assert!(html_content_page02.contains(&contracts[150].address));
        assert!(html_content_page02.contains(&contracts[150].name));
        assert!(html_content_page02.contains(&contracts[150].compiler));
        assert!(html_content_page02.contains(&contracts[150].compiler_version));
        
        assert!(html_content_page03.contains(&contracts[250].address));
        assert!(html_content_page03.contains(&contracts[250].name));
        assert!(html_content_page03.contains(&contracts[250].compiler));
        assert!(html_content_page03.contains(&contracts[250].compiler_version));
        
        assert!(html_content_page04.contains(&contracts[350].address));
        assert!(html_content_page04.contains(&contracts[350].name));
        assert!(html_content_page04.contains(&contracts[350].compiler));
        assert!(html_content_page04.contains(&contracts[350].compiler_version));
        
        assert!(html_content_page05.contains(&contracts[450].address));
        assert!(html_content_page05.contains(&contracts[450].name));
        assert!(html_content_page05.contains(&contracts[450].compiler));
        assert!(html_content_page05.contains(&contracts[450].compiler_version));
    }
}
