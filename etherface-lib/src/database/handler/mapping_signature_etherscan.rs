use crate::database::schema::mapping_signature_etherscan;
use crate::model::MappingSignatureEtherscan;
// use crate::database::schema::mapping_signature_etherscan::dsl::*;

use diesel::prelude::*;
use diesel::PgConnection;

pub struct MappingSignatureEtherscanHandler<'a> {
    connection: &'a PgConnection,
}

impl<'a> MappingSignatureEtherscanHandler<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        MappingSignatureEtherscanHandler { connection }
    }

    pub fn insert(&self, entity: &MappingSignatureEtherscan) -> usize {
        diesel::insert_into(mapping_signature_etherscan::table)
            .values(entity)
            .on_conflict_do_nothing()
            .execute(self.connection)
            .unwrap()
    }
}
