#![allow(unused)]
use crate::models::*;
use crate::schema::devices::dsl::*;
use crate::schema::domains::dsl::*;
use crate::schema::users::dsl::*;
use anyhow::Result;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, GroupedBy, OptionalExtension,
    QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
};
use uuid::Uuid;
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
    pub fn create_user(&mut self, u_name: &str, p_hash: &str) -> Result<User> {
        let mut conn = self.pool.get()?;
        let new_user = NewUser { username: u_name, password_hash: p_hash };
        let user = diesel::insert_into(users).values(&new_user).get_result(&mut conn)?;
        Ok(user)
    }

    pub fn get_all_users(&mut self) -> Result<Vec<String>> {
        let mut conn = self.pool.get()?;
        let all_users = users.select(username).load::<(String)>(&mut conn)?;
        Ok(all_users)
    }

    pub fn find_user_by_username(&mut self, in_name: &str) -> Result<Option<User>> {
        let mut conn = self.pool.get()?;
        let result = users.filter(username.eq(in_name)).first::<User>(&mut conn).optional()?;
        Ok(result)
    }

    pub fn delete_user_by_username(&mut self, in_name: &str) -> Result<usize> {
        //TODO: 添加使用者檢查，不要寫在外面
        let mut conn = self.pool.get()?;
        let count = diesel::delete(users.filter(username.eq(in_name))).execute(&mut conn)?;
        Ok(count)
    }

    pub fn get_all_devices(&mut self) -> Result<Vec<String>> {
        let mut conn = self.pool.get()?;
        let all_devices = devices.select(device_name).load::<(String)>(&mut conn)?;
        Ok(all_devices)
    }

    pub fn get_user_all_devices(&mut self, u_s: &[User]) -> Result<Vec<Vec<Device>>> {
        let mut conn = self.pool.get()?;
        let devices_list = Device::belonging_to(u_s)
            .select(Device::as_select())
            .load::<crate::models::Device>(&mut conn)?;
        let grouped_devices = devices_list.grouped_by(u_s);
        Ok(grouped_devices)
    }
    //Device operations
    pub fn create_device(
        &mut self,
        u_name: &str,
        d_id: Uuid,
        d_name: String,
        t_hash: String,
    ) -> Result<Device> {
        let user = self.find_user_by_username(u_name)?.ok_or(anyhow::anyhow!("User not found"))?;
        let mut conn = self.pool.get()?;
        let new_device = NewDevice {
            user_id: user.id,
            device_identifier: d_id.to_string(),
            token_hash: t_hash,
            device_name: d_name,
            updated_at: chrono::Utc::now().naive_utc(),
        };
        let device = diesel::insert_into(devices).values(&new_device).get_result(&mut conn)?;
        Ok(device)
    }

    pub fn find_by_device_identifier(&mut self, ident: &str) -> Result<Option<Device>> {
        let mut conn = self.pool.get()?;
        let result =
            devices.filter(device_identifier.eq(ident)).first::<Device>(&mut conn).optional()?;
        Ok(result)
    }

    pub fn get_device_all_domains(&mut self, devs: &[Device]) -> Result<Vec<Vec<Domain>>> {
        let mut conn = self.pool.get()?;
        let domains_list = Domain::belonging_to(devs)
            .select(Domain::as_select())
            .load::<crate::models::Domain>(&mut conn)?;
        let grouped_domains = domains_list.grouped_by(devs);
        Ok(grouped_domains)
    }

    //Domain operations
    pub fn create_domain(&mut self, dev_id: i32, host: &str, is_a: bool) -> Result<Domain> {
        let mut conn = self.pool.get()?;
        let new_domain = NewDomain {
            device_id: dev_id,
            hostname: host.to_string(),
            is_active: is_a,
            updated_at: chrono::Utc::now().naive_utc(),
        };
        let domain = diesel::insert_into(domains).values(&new_domain).get_result(&mut conn)?;
        Ok(domain)
    }

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
    use diesel::{r2d2::CustomizeConnection, sql_query};
    use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
    use uuid::Uuid;
    #[derive(Debug)]
    pub struct SqliteCustomizer;

    impl CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteCustomizer {
        fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
            sql_query("PRAGMA foreign_keys = ON;")
                .execute(conn)
                .map_err(diesel::r2d2::Error::QueryError)?;

            Ok(())
        }

        fn on_release(&self, _conn: SqliteConnection) {}
    }
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
    fn setup_test_service() -> DbService {
        let db_url = format!("file:memdb{}?mode=memory&cache=shared", uuid::Uuid::new_v4());
        let manager = ConnectionManager::<SqliteConnection>::new(db_url);
        let pool = Pool::builder()
            .connection_customizer(Box::new(SqliteCustomizer))
            .max_size(1)
            .build(manager)
            .expect("Failed to create pool");

        let mut conn = pool.get().expect("Failed to get conn");
        conn.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");
        DbService::new(pool)
    }

    #[test]
    fn test_create_user() -> Result<()> {
        let mut service = setup_test_service();
        let user = service.create_user("test", "password_hash")?;
        assert_eq!(user.username, "test");
        assert_eq!(user.password_hash, "password_hash");
        Ok(())
    }

    #[test]
    fn test_find_user_by_username() -> Result<()> {
        let mut service = setup_test_service();
        service.create_user("test", "password_hash")?;
        let result = service.find_user_by_username("test")?;
        assert!(result.is_some());
        let user = result.unwrap();
        assert_eq!(user.username, "test");
        assert_eq!(user.password_hash, "password_hash");
        Ok(())
    }

    #[test]
    fn test_delete_user_by_username() -> Result<()> {
        let mut service = setup_test_service();
        service.create_user("test", "password_hash")?;
        let count = service.delete_user_by_username("test")?;
        assert_eq!(count, 1);
        let result = service.find_user_by_username("test")?;
        assert!(result.is_none());
        Ok(())
    }

    #[test]
    fn test_create_device() -> Result<()> {
        let mut service = setup_test_service();
        let user = service.create_user("test_user", "password_hash")?;
        let device_iden = Uuid::new_v4();
        let device = service.create_device(
            "test_user",
            device_iden,
            "test_device".into(),
            "tokenhash".into(),
        )?;
        assert_eq!(device.user_id, user.id);
        assert_eq!(device.device_identifier, device_iden.to_string());
        assert_eq!(device.token_hash, "tokenhash");
        Ok(())
    }

    #[test]
    fn test_find_by_device_identifier() -> Result<()> {
        let mut service = setup_test_service();
        let device_iden = Uuid::new_v4();
        service.create_user("test_user", "password_hash")?;
        service.create_device(
            "test_user",
            device_iden,
            "test_device".into(),
            "tokenhash".into(),
        )?;
        let result = service.find_by_device_identifier(&device_iden.to_string())?;
        assert!(result.is_some());
        let device = result.unwrap();
        assert_eq!(device.device_identifier, device_iden.to_string());
        Ok(())
    }

    #[test]
    fn test_get_user_all_devices() -> Result<()> {
        let mut service = setup_test_service();
        let device_iden = Uuid::new_v4();
        let user = service.create_user("test_user", "password_hash")?;
        let device = service.create_device(
            "test_user",
            device_iden,
            "test_device".into(),
            "tokenhash".into(),
        )?;
        let users_list = {
            let mut conn = service.pool.get()?;
            users.load::<User>(&mut conn)?
        };
        let devices_grouped = service.get_user_all_devices(&users_list)?;
        assert_eq!(devices_grouped.len(), 1);
        assert_eq!(devices_grouped[0].len(), 1);
        assert_eq!(devices_grouped[0][0].device_identifier, device_iden.to_string());
        Ok(())
    }

    #[test]
    fn test_create_domain() -> Result<()> {
        let mut service = setup_test_service();
        let device_iden = Uuid::new_v4();
        let user = service.create_user("test_user", "password_hash")?;
        let device = service.create_device(
            "test_user",
            device_iden,
            "test_device".into(),
            "tokenhash".into(),
        )?;
        let domain = service.create_domain(device.id, "example.com", true)?;
        assert_eq!(domain.device_id, device.id);
        assert_eq!(domain.hostname, "example.com");
        assert!(domain.is_active);
        Ok(())
    }

    #[test]
    fn test_find_active_domains_by_device_id() -> Result<()> {
        let mut service = setup_test_service();
        let device_iden = Uuid::new_v4();
        let user = service.create_user("test_user", "password_hash")?;
        let device = service.create_device(
            "test_user",
            device_iden,
            "test_device".into(),
            "tokenhash".into(),
        )?;
        service.create_domain(device.id, "example.com", true)?;
        service.create_domain(device.id, "inactive.com", false)?;
        let active_domains = service.find_active_domains_by_device_id(device.id)?;
        assert_eq!(active_domains.len(), 1);
        assert_eq!(active_domains[0].hostname, "example.com");
        Ok(())
    }

    #[test]
    fn test_get_device_all_domains() -> Result<()> {
        let mut service = setup_test_service();
        let device_iden = Uuid::new_v4();
        let user = service.create_user("test_user", "password_hash")?;
        let device = service.create_device(
            "test_user",
            device_iden,
            "test_device".into(),
            "tokenhash".into(),
        )?;
        let domain1 = service.create_domain(device.id, "example.com", true)?;
        let domain2 = service.create_domain(device.id, "inactive.com", false)?;
        let devices_list = {
            let mut conn = service.pool.get()?;
            devices.load::<Device>(&mut conn)?
        };
        let domains_grouped = service.get_device_all_domains(&devices_list)?;
        assert_eq!(domains_grouped.len(), 1);
        assert_eq!(domains_grouped[0].len(), 2);
        assert_eq!(domains_grouped[0][0].hostname, "example.com");
        assert_eq!(domains_grouped[0][1].hostname, "inactive.com");
        Ok(())
    }
}
