use crate::database::schema::mapping_signature_fourbyte;
use crate::database::schema::mapping_signature_fourbyte::dsl::*;
use crate::model::MappingSignatureFourbyte;
use crate::model::SignatureKind;
use diesel::prelude::*;
use diesel::PgConnection;

pub struct MappingSignatureFourbyteHandler<'a> {
    connection: &'a PgConnection,
}

impl<'a> MappingSignatureFourbyteHandler<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        MappingSignatureFourbyteHandler { connection }
    }

    pub fn get(&self, entity: &MappingSignatureFourbyte) -> Option<MappingSignatureFourbyte> {
        mapping_signature_fourbyte
            .filter(signature_id.eq(&entity.signature_id).and(kind.eq(&entity.kind)))
            .first(self.connection)
            .optional()
            .unwrap()
    }

    pub fn get_functions_count(&self) -> usize {
        mapping_signature_fourbyte.filter(kind.eq(SignatureKind::Function)).execute(self.connection).unwrap()
    }

    pub fn get_events_count(&self) -> usize {
        mapping_signature_fourbyte.filter(kind.eq(SignatureKind::Event)).execute(self.connection).unwrap()
    }

    pub fn insert(&self, entity: &MappingSignatureFourbyte) {
        diesel::insert_into(mapping_signature_fourbyte::table)
            .values(entity)
            .on_conflict_do_nothing()
            .execute(self.connection)
            .unwrap();
    }
}
