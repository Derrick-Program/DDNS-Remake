use nanoid::nanoid;
use argon2::{
    Argon2, PasswordHash, PasswordVerifier, password_hash::{PasswordHasher, SaltString, rand_core::OsRng}
};

fn generate_api_key() -> String {
    let token = nanoid!(45);
    format!("ddns_tok_{}", token)
}

fn hash_token(token: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(token.as_bytes(), &salt).unwrap().to_string()
}

fn verify_client_token(db_hash: &str, provided_token: &str) -> bool {
    let parsed_hash = match PasswordHash::new(db_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };
    let argon2 = Argon2::default();
    argon2.verify_password(provided_token.as_bytes(), &parsed_hash).is_ok()
}

pub fn generate_and_print_api_key() {
    let api_key = generate_api_key();
    println!("Generated API Key: {}", api_key);
    let db_token = hash_token(&api_key);
    println!("Hashed API Key for DB storage: {}", db_token);
    let is_valid = verify_client_token(&db_token, &api_key);
    println!("Token verification result: {}", is_valid);
}