use std::sync::OnceLock;

use salvo::{jwt_auth::{ConstDecoder, HeaderFinder}, prelude::JwtAuth};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JwtClaims {
    pub uid: i32,
    pub username: String,
    pub exp: i64,
}

static JWT_SECRET: OnceLock<String> = OnceLock::new();

pub fn get_secret() -> &'static [u8] {
    JWT_SECRET.get_or_init(|| nanoid::nanoid!(32)).as_bytes()
}

pub fn jwt_middleware() -> JwtAuth<JwtClaims, ConstDecoder> {
    JwtAuth::new(ConstDecoder::from_secret(get_secret()))
        .finders(vec![Box::new(HeaderFinder::new())])
}