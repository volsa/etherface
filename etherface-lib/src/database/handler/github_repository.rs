//! `github_repository` table handler.

use crate::database::schema::github_repository;
use crate::database::schema::github_repository::dsl::*;
use crate::model::GithubRepository;
use crate::model::GithubRepositoryDatabase;
use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::PgConnection;
use diesel::RunQueryDsl;
use log::debug;

pub struct GithubRepositoryHandler<'a> {
    connection: &'a PgConnection,
}

impl<'a> GithubRepositoryHandler<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        GithubRepositoryHandler { connection }
    }

    pub fn get_total_count(&self) -> i64 {
        github_repository.count().get_result(self.connection).unwrap()
    }

    pub fn insert(&self, entity: &GithubRepository, entity_solidity_ratio: f32, by_crawling: bool) {
        diesel::insert_into(github_repository::table)
            .values(&entity.to_insertable(Some(entity_solidity_ratio), by_crawling))
            .execute(self.connection)
            .unwrap();
    }

    pub fn update(&self, entity: &GithubRepository, entity_ratio: f32) {
        diesel::update(github_repository.filter(id.eq(entity.id)))
            .set((
                name.eq(&entity.name),
                html_url.eq(&entity.html_url),
                language.eq(&entity.language),
                stargazers_count.eq(entity.stargazers_count),
                size.eq(entity.size),
                pushed_at.eq(entity.pushed_at),
                updated_at.eq(entity.updated_at),
                solidity_ratio.eq(Some(entity_ratio)),
            ))
            .execute(self.connection)
            .unwrap();
    }

    pub fn update_and_set_scraped_to_null(&self, entity: &GithubRepository, entity_solidity_ratio: f32) {
        diesel::update(github_repository.filter(id.eq(entity.id)))
            .set((
                name.eq(&entity.name),
                html_url.eq(&entity.html_url),
                language.eq(&entity.language),
                pushed_at.eq(&entity.pushed_at),
                updated_at.eq(&entity.updated_at),
                solidity_ratio.eq(&entity_solidity_ratio),
                visited_at.eq(Some(Utc::now())),
                scraped_at.eq::<Option<DateTime<Utc>>>(None), // Set to NULL to trigger re-scraping
            ))
            .execute(self.connection)
            .unwrap();
    }

    pub fn get_unvisited_ordered_by_added_at(&self) -> Vec<GithubRepositoryDatabase> {
        sql_query(
            "SELECT github_repository.* FROM github_repository 
            JOIN mapping_signature_github ON github_repository.id = mapping_signature_github.repository_id
            WHERE 
                (github_repository.solidity_ratio > 0.0 OR github_repository.language LIKE 'Solidity') 
                AND github_repository.visited_at IS NULL 
                AND github_repository.is_deleted IS FALSE 
                AND github_repository.fork IS FALSE
            GROUP BY github_repository.id 
            ORDER BY github_repository.added_at DESC",
        )
        .load(self.connection)
        .unwrap()
    }

    pub fn get_unvisited_ordered_by_signature_count(&self) -> Vec<GithubRepositoryDatabase> {
        sql_query(
            "SELECT github_repository.* FROM github_repository 
            JOIN mapping_signature_github ON github_repository.id = mapping_signature_github.repository_id
            WHERE 
                (github_repository.solidity_ratio > 0.0 OR github_repository.language LIKE 'Solidity') 
                AND github_repository.visited_at IS NULL 
                AND github_repository.is_deleted IS FALSE 
                AND github_repository.fork IS FALSE
            GROUP BY github_repository.id 
            ORDER BY COUNT(*) DESC",
        )
        .load(self.connection)
        .unwrap()
    }

    pub fn set_ratio(&self, entity_id: i32, entity_ratio: f32) {
        diesel::update(github_repository.filter(id.eq(entity_id)))
            .set(solidity_ratio.eq(entity_ratio))
            .execute(self.connection)
            .unwrap();
    }

    /// Sets the `github_repository::scraped_at` field to NULL in order to re-trigger the scraping process.
    pub fn set_scraped_to_null(&self, entity_id: i32) {
        diesel::update(github_repository.filter(id.eq(entity_id)))
            .set(scraped_at.eq::<Option<DateTime<Utc>>>(None))
            .execute(self.connection)
            .unwrap();
    }

    pub fn get_total_repo_count_of_user(&self, entity_id: i32) -> i64 {
        github_repository.filter(id.eq(entity_id)).count().get_result(self.connection).unwrap()
    }

    pub fn get_solidity_repo_count_of_user(&self, entity_id: i32) -> i64 {
        github_repository
            .filter(id.eq(entity_id).and(solidity_ratio.gt(0.0)))
            .count()
            .get_result(self.connection)
            .unwrap()
    }

    pub fn get_solidity_repos_active_in_last_n_days(&self, days: i64) -> Vec<GithubRepositoryDatabase> {
        github_repository
            .filter(
                updated_at
                    .gt(Utc::now() - chrono::Duration::days(days))
                    .and(solidity_ratio.gt(0.0).or(language.eq("Solidity"))),
            )
            .get_results(self.connection)
            .unwrap()
    }

    pub fn get_unvisited(&self) -> Vec<GithubRepositoryDatabase> {
        github_repository
            .filter(visited_at.is_null().and(solidity_ratio.gt(0.0)))
            .get_results(self.connection)
            .unwrap()
    }

    pub fn get_unscraped_with_forks(&self) -> Vec<GithubRepositoryDatabase> {
        github_repository
            .filter(scraped_at.is_null().and(is_deleted.eq(false)).and(solidity_ratio.gt(0.0)))
            .get_results(self.connection)
            .unwrap()
    }

    pub fn get_unscraped_without_forks(&self) -> Vec<GithubRepositoryDatabase> {
        github_repository
            .filter(
                scraped_at
                    .is_null()
                    .and(is_deleted.eq(false))
                    .and(solidity_ratio.gt(0.0))
                    .and(fork.eq(false)),
            )
            .get_results(self.connection)
            .unwrap()
    }

    pub fn set_visited(&self, entity_id: i32) {
        diesel::update(github_repository.filter(id.eq(entity_id)))
            .set(visited_at.eq(Utc::now()))
            .execute(self.connection)
            .unwrap();
    }

    pub fn set_scraped(&self, entity_id: i32) {
        diesel::update(github_repository.filter(id.eq(entity_id)))
            .set(scraped_at.eq(Utc::now()))
            .execute(self.connection)
            .unwrap();
    }

    // pub fn set_solidity_ratio(&self, entity_id: i32, entity_solidity_ratio: f32) {
    //     diesel::update(github_repository.filter(id.eq(entity_id)))
    //         .set(solidity_ratio.eq(entity_solidity_ratio))
    //         .execute(self.connection)
    //         .unwrap();
    // }

    pub fn set_deleted(&self, entity_id: i32) {
        diesel::update(github_repository.filter(id.eq(entity_id)))
            .set(is_deleted.eq(true))
            .execute(self.connection)
            .unwrap();
        debug!("Setting repository with id '{entity_id}' as deleted");
    }

    pub fn set_undeleted(&self, entity_id: i32) {
        diesel::update(github_repository.filter(id.eq(entity_id)))
            .set(is_deleted.eq(false))
            .execute(self.connection)
            .unwrap();
    }

    pub fn get_by_id(&self, entity_id: i32) -> Option<GithubRepositoryDatabase> {
        github_repository.filter(id.eq(entity_id)).get_result(self.connection).optional().unwrap()
    }

    pub fn get_unvisited_repos_with_ratio_greater_than(&self, ratio: f32) -> Vec<GithubRepositoryDatabase> {
        github_repository
            .filter(
                github_repository::visited_at
                    .is_null()
                    .and(github_repository::fork.eq(false))
                    .and(github_repository::solidity_ratio.gt(ratio)),
            )
            .distinct_on(github_repository::id)
            .select(github_repository::all_columns)
            .load(self.connection)
            .unwrap()
    }
}
