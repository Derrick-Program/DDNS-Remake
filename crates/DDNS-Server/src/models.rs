use diesel::prelude::*;
use crate::schema::{users, devices, domains};
use chrono::NaiveDateTime; 

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub password_hash: &'a str,
}


#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(belongs_to(User))] 
#[diesel(table_name = devices)]
pub struct Device {
    pub id: i32,
    pub user_id: i32,
    pub device_identifier: String,
    pub token_hash: String,
    pub last_seen_ip: Option<String>, 
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = devices)]
pub struct NewDevice {
    pub user_id: i32,
    pub device_identifier: String,
    pub token_hash: String,
    pub updated_at: NaiveDateTime,
}


#[derive(Queryable, Selectable, Identifiable, Associations, Debug)]
#[diesel(belongs_to(Device))] 
#[diesel(table_name = domains)]
pub struct Domain {
    pub id: i32,
    pub device_id: i32,
    pub hostname: String,
    pub current_ip: Option<String>, 
    pub is_active: bool,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = domains)]
pub struct NewDomain {
    pub device_id: i32,
    pub hostname: String,
    pub is_active: bool,
    pub updated_at: NaiveDateTime,
}