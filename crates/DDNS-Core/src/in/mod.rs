use salvo::oapi::ToSchema;
use serde::Deserialize;

#[derive(Deserialize, ToSchema, Debug)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
}