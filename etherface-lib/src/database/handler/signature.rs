use crate::database::schema::signature;
use crate::database::schema::signature::dsl::*;
use crate::model::Signature;
use crate::model::SignatureWithMetadata;
use diesel::prelude::*;
use diesel::PgConnection;

pub struct SignatureHandler<'a> {
    connection: &'a PgConnection,
}

impl<'a> SignatureHandler<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        SignatureHandler { connection }
    }

    pub fn get_latest_500(&self) -> Vec<Signature> {
        signature
            .select(signature::table::all_columns())
            .limit(500)
            .order_by(id.desc())
            .get_results(self.connection)
            .unwrap()
    }

    pub fn insert(&self, entity: &SignatureWithMetadata) -> Signature {
        if let Some(val) = self.get_by_hash(&entity.hash) {
            return val;
        }

        diesel::insert_into(signature::table)
            .values(&entity.to_insertable())
            .get_result(self.connection)
            .unwrap()
    }

    fn get_by_hash(&self, entity_hash: &str) -> Option<Signature> {
        signature.filter(hash.eq(entity_hash)).first(self.connection).optional().unwrap()
    }
}
