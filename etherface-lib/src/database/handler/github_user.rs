use crate::database::schema::github_user;
use crate::database::schema::github_user::dsl::*;
use crate::model::GithubUser;
use crate::model::GithubUserDatabase;
use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel::RunQueryDsl;

pub struct GithubUserHandler<'a> {
    connection: &'a PgConnection,
}

impl<'a> GithubUserHandler<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        GithubUserHandler { connection }
    }

    pub fn insert_if_not_exists(&self, entity: &GithubUser) {
        diesel::insert_into(github_user::table)
            .values(entity.to_insertable())
            .on_conflict_do_nothing()
            .execute(self.connection)
            .unwrap();
    }

    pub fn get_by_id(&self, entity_id: i32) -> GithubUserDatabase {
        github_user.filter(id.eq(entity_id)).get_result(self.connection).unwrap()
    }

    pub fn repo_count(&self, entity_id: i32) -> i64 {
        use crate::database::schema::github_repository;

        github_user
            .inner_join(github_repository::table)
            .filter(github_user::id.eq(entity_id).and(github_repository::is_deleted.eq(false)))
            .count()
            .get_result(self.connection)
            .unwrap()
    }

    pub fn get_unvisited_solidity_repository_owners_orderd_by_added_at(&self) -> Vec<GithubUserDatabase> {
        use crate::database::schema::github_repository;

        github_user
            .inner_join(github_repository::table)
            .filter(
                (github_repository::solidity_ratio.gt(0.0).or(github_repository::language.eq("Solidity")))
                    .and(github_user::visited_at.is_null()),
            )
            .select(github_user::all_columns)
            .order_by(github_user::added_at.desc())
            .load(self.connection)
            .unwrap()
    }

    pub fn set_deleted(&self, entity_id: i32) {
        diesel::update(github_user.filter(id.eq(entity_id)))
            .set(is_deleted.eq(true))
            .execute(self.connection)
            .unwrap();
    }

    pub fn get_solidity_repository_owners_active_in_last_n_days(&self, days: i64) -> Vec<GithubUserDatabase> {
        use crate::database::schema::github_repository;

        github_user
            .inner_join(github_repository::table)
            .filter(
                (github_repository::solidity_ratio.gt(0.0).or(github_repository::language.eq("Solidity")))
                    .and(
                        github_repository::is_deleted
                            .eq(false)
                            .and(github_repository::updated_at.gt(Utc::now() - chrono::Duration::days(days))),
                    ),
            )
            .select(github_user::all_columns)
            .distinct()
            .load(self.connection)
            .unwrap()
    }

    pub fn set_visited(&self, entity_id: i32) {
        diesel::update(github_user::table)
            .filter(id.eq(entity_id))
            .set(visited_at.eq(Utc::now()))
            .execute(self.connection)
            .unwrap();
    }
}
