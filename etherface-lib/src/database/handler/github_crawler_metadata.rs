// use crate::database::schema::github_crawler_metadata;
use crate::database::schema::github_crawler_metadata::dsl::*;
use crate::model::GithubCrawlerMetadata;
use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection;

pub struct GithubCrawlerMetadataHandler<'a> {
    connection: &'a PgConnection,
}

impl<'a> GithubCrawlerMetadataHandler<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        GithubCrawlerMetadataHandler { connection }
    }

    pub fn get(&self) -> GithubCrawlerMetadata {
        // In theory we _should_ only have one entry with ID == 1 in our database, which gets created when the
        // initial migration is executed.
        github_crawler_metadata.filter(id.eq(1)).get_result(self.connection).unwrap()
    }

    pub fn update_last_repository_search_date(&self, date: DateTime<Utc>) {
        diesel::update(github_crawler_metadata.filter(id.eq(1)))
            .set(last_repository_search.eq(date))
            .execute(self.connection)
            .unwrap();
    }

    pub fn update_last_repository_check_date(&self, date: DateTime<Utc>) {
        diesel::update(github_crawler_metadata.filter(id.eq(1)))
            .set(last_repository_check.eq(date))
            .execute(self.connection)
            .unwrap();
    }

    pub fn update_last_user_check_date(&self, date: DateTime<Utc>) {
        diesel::update(github_crawler_metadata.filter(id.eq(1)))
            .set(last_user_check.eq(date))
            .execute(self.connection)
            .unwrap();
    }
}
