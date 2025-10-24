use std::collections::HashMap;

use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use chrono::Duration;
use serde::Serialize;
use sqlx::{PgPool, Row};
use time::PrimitiveDateTime;
use crate::{jwt::generate_token, user_struct::{CreateUser, User, UserPublicInfo}};

#[derive(Serialize)]
pub struct Token {
    pub user_id: i32,
    pub refresh_token: String,
    pub access_token: String,
}

pub async fn signup_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Token>, (StatusCode, String)> {
    let name = payload.name;
    let raw_pass = payload.pass;

    let row = sqlx::query!("SELECT id, name FROM users WHERE name = $1", name)
        .fetch_optional(&pool)
        .await
        .unwrap();

    if let Some(row) = row {
        if name == row.name.unwrap() {
            return Err((StatusCode::CONFLICT, "Same name".into()));
        }
    }

    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();

    let hashed = argon2.hash_password(raw_pass.as_bytes(), &salt).unwrap();

    #[cfg(debug_assertions)]
    println!("SIGN UP: {hashed:?}");

    sqlx::query!("INSERT INTO users (name, pass_hash) VALUES ($1, $2)", name, hashed.to_string())
        .execute(&pool)
        .await.unwrap();

    let new_user_record = sqlx::query!("SELECT id FROM users WHERE name = $1", name)
        .fetch_optional(&pool)
        .await.unwrap().unwrap();

    let key = std::env::var("JWT_KEY").expect("JWT_KEY not set in .env");
    Ok(Json(Token {
        user_id: new_user_record.id,
        refresh_token: generate_token(name.clone(), key.clone(), Duration::days(182)),
        access_token: generate_token(name.clone(), key.clone(), Duration::minutes(30)),
    }))
}

pub async fn login_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Token>, (StatusCode, String)> {
    let name = payload.name;

    let row = sqlx::query!("SELECT id, name, pass_hash FROM users WHERE name = $1", name)
        .fetch_optional(&pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error".into()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid username".into()))?;

    let parsed_hash = PasswordHash::new(&row.pass_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid stored hash".into()))?;

    let argon2 = Argon2::default();

    let _ = argon2
        .verify_password(payload.pass.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Wrong password".into()))?;

    let key = std::env::var("JWT_KEY").expect("JWT_KEY not set in .env");
    Ok(Json(Token {
        user_id: row.id,
        refresh_token: generate_token(name.clone(), key.clone(), Duration::days(182)),
        access_token: generate_token(name.clone(), key.clone(), Duration::minutes(30)),
    }))
}

pub async fn get_user(
    State(pool): State<PgPool>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<UserPublicInfo>, (StatusCode, String)> {
    let maybe_user = if let Some(id) = params.get("id") {
        let id: i32 = id.parse::<i32>().map_err(|_| (StatusCode::BAD_REQUEST, "Invalid id".to_string()))?;
        sqlx::query("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
    } else if let Some(name) = params.get("name").or_else(|| params.get("user")) {
        sqlx::query("SELECT * FROM users WHERE name = $1")
            .bind(name)
            .fetch_one(&pool)
            .await
    } else {
        return Err((StatusCode::BAD_REQUEST, "No query parameter provided".to_string()));
    };

    match maybe_user {
        Ok(user) => Ok(Json(UserPublicInfo {
            id: user.get::<i32, _>("id"),
            name: user.get::<String, _>("name"),
            joined_at: user.get::<PrimitiveDateTime, _>("joined_at"),
        })),
        Err(sqlx::Error::RowNotFound) => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}
