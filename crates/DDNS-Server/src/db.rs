use crate::models::*;
use anyhow::Result;
use diesel::{ExpressionMethods,OptionalExtension,QueryDsl,RunQueryDsl,SqliteConnection};
use diesel::r2d2::{ConnectionManager,Pool};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Clone)]
pub struct DbService{
    pool: DbPool,
}

impl DbService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
    pub fn find_by_identifier(&mut self, ident: &str) -> Result<Option<Device>> {
        use crate::schema::devices::dsl::*;
        let mut conn = self.pool.get()?;
        let result =
            devices.filter(device_identifier.eq(ident)).first::<Device>(&mut conn).optional()?;
        Ok(result)
    }
}
