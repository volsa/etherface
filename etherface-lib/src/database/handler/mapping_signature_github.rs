use crate::database::schema::mapping_signature_github;
use crate::model::MappingSignatureGithub;
// use crate::database::schema::mapping_signature_github::dsl::*;

use diesel::prelude::*;
use diesel::PgConnection;

pub struct MappingSignatureGithubHandler<'a> {
    connection: &'a PgConnection,
}

impl<'a> MappingSignatureGithubHandler<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        MappingSignatureGithubHandler { connection }
    }

    pub fn insert(&self, entity: &MappingSignatureGithub) {
        diesel::insert_into(mapping_signature_github::table)
            .values(entity)
            .on_conflict_do_nothing()
            .execute(self.connection)
            .unwrap();
    }
}
