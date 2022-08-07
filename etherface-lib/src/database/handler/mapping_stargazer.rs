use crate::database::schema::mapping_stargazer;
use crate::model::MappingStargazer;
// use crate::database::schema::mapping_stargazer::dsl::*;

use diesel::prelude::*;
use diesel::PgConnection;

pub struct MappingStargazerHandler<'a> {
    connection: &'a PgConnection,
}

impl<'a> MappingStargazerHandler<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        MappingStargazerHandler { connection }
    }

    pub fn insert(&self, entity: &MappingStargazer) {
        diesel::insert_into(mapping_stargazer::table)
            .values(entity)
            .on_conflict_do_nothing()
            .execute(self.connection)
            .unwrap();
    }
}
