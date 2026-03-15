use crate::models::*;
use crate::schema::devices::dsl::*;
use crate::schema::domains::dsl::*;
use crate::schema::users::dsl::*;
use anyhow::Result;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl,
    SqliteConnection,
};
pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Clone)]
pub struct DbService {
    pool: DbPool,
}

impl DbService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    //User operations
    pub fn find_user_by_username(&mut self, in_name: &str) -> Result<Option<User>> {
        let mut conn = self.pool.get()?;
        let result = users.filter(username.eq(in_name)).first::<User>(&mut conn).optional()?;
        Ok(result)
    }

    //Device operations
    pub fn find_by_identifier(&mut self, ident: &str) -> Result<Option<Device>> {
        let mut conn = self.pool.get()?;
        let result =
            devices.filter(device_identifier.eq(ident)).first::<Device>(&mut conn).optional()?;
        Ok(result)
    }

    //Domain operations
    pub fn find_active_domains_by_device_id(&mut self, dev_id: i32) -> Result<Vec<Domain>> {
        let mut conn = self.pool.get()?;
        let results = domains
            .filter(device_id.eq(dev_id).and(is_active.eq(true)))
            .load::<Domain>(&mut conn)?;
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
    fn setup_test_service() -> DbService {
        let manager = ConnectionManager::<SqliteConnection>::new("file::memory:?cache=shared");
        let pool = Pool::builder().max_size(1).build(manager).expect("Failed to create pool");

        let mut conn = pool.get().expect("Failed to get conn");
        conn.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");
        DbService::new(pool)
    }

    #[test]
    fn test_with_user_table() -> Result<()> {
        let mut service = setup_test_service();
        {
            let mut conn = service.pool.get()?;
            diesel::insert_into(users)
                .values(NewUser { username: "test".into(), password_hash: "hash".into() })
                .execute(&mut conn)?;
        }
        let results = service.find_user_by_username("test")?;
        assert!(results.is_some());
        let user = results.unwrap();
        assert_eq!(user.username, "test");
        assert_eq!(user.password_hash, "hash");
        Ok(())
    }
}
