use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;

use crate::http::Token;

#[derive(Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub pass: String,
}

#[derive(Serialize)]
pub struct User {
    id: i32,
    room_id: i32,
    name: String,
    pass_hash: String,
    joined_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
pub struct UserPublicInfo {
    pub id: i32,
    pub name: String,
    pub joined_at: PrimitiveDateTime,
}
