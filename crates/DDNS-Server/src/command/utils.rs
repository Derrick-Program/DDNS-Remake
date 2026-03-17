#![allow(unused)]
use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use nanoid::nanoid;
use uuid::Uuid;

pub fn generate_api_key() -> String {
    let token = nanoid!(45);
    format!("ddns_tok_{}", token)
}

pub fn hash_token(token: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(token.as_bytes(), &salt).unwrap().to_string()
}

pub fn verify_client_token(db_hash: &str, provided_token: &str) -> bool {
    let parsed_hash = match PasswordHash::new(db_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };
    let argon2 = Argon2::default();
    argon2.verify_password(provided_token.as_bytes(), &parsed_hash).is_ok()
}
const MY_SYSTEM_NAMESPACE: Uuid = uuid::uuid!("c2139751-7d74-4ae1-b413-54c0155ea5aa");

fn get_device_id() -> Result<Uuid, String> {
    let machine_id_string =
        machine_uid::get().map_err(|e| format!("無法獲取系統 Machine ID: {:?}", e))?;

    let device_uuid = Uuid::new_v5(&MY_SYSTEM_NAMESPACE, machine_id_string.as_bytes());
    Ok(device_uuid)
}

pub fn generate_and_print_api_key() {
    let api_key = generate_api_key();
    println!("Generated API Key: {}", api_key);
    let db_token = hash_token(&api_key);
    println!("Hashed API Key for DB storage: {}", db_token);
    let is_valid = verify_client_token(&db_token, &api_key);
    println!("Token verification result: {}", is_valid);
    let host_uuid = uuid::Uuid::new_v4();
    println!("Generated Host UUID: {}", host_uuid);
}
