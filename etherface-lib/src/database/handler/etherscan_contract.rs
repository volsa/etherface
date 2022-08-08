use crate::database::schema::etherscan_contract;
use crate::database::schema::etherscan_contract::dsl::*;
use crate::model::EtherscanContract;
use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection;

pub struct EtherscanContractHandler<'a> {
    connection: &'a PgConnection,
}

impl<'a> EtherscanContractHandler<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        EtherscanContractHandler { connection }
    }

    pub fn insert(&self, entity: &EtherscanContract) -> EtherscanContract {
        if let Some(row) = self.get(entity) {
            return row;
        }

        diesel::insert_into(etherscan_contract::table)
            .values(&entity.to_insertable())
            .get_result(self.connection)
            .unwrap()
    }

    fn get(&self, entity: &EtherscanContract) -> Option<EtherscanContract> {
        etherscan_contract.filter(address.eq(&entity.address)).first(self.connection).optional().unwrap()
    }

    pub fn get_unvisited(&self) -> Vec<EtherscanContract> {
        etherscan_contract.filter(scraped_at.is_null()).get_results(self.connection).unwrap()
    }

    pub fn set_visited(&self, entity: &EtherscanContract) {
        diesel::update(etherscan_contract.filter(address.eq(&entity.address)))
            .set(scraped_at.eq(Utc::now()))
            .execute(self.connection)
            .unwrap();
    }
}
