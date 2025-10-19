use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;
use crate::user_struct::CreateUser;

pub async fn signup_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>, 
) -> Result<StatusCode, (StatusCode, String)> {
    let name = payload.name;
    let raw_pass = payload.pass;

    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();

    let hashed = argon2.hash_password(raw_pass.as_bytes(), &salt).unwrap();

    #[cfg(debug_assertions)]
    println!("SIGN UP: {hashed:?}");

    sqlx::query!("INSERT INTO users (name, pass_hash) VALUES ($1, $2)", name, hashed.to_string())
        .execute(&pool)
        .await.unwrap();

    Ok(StatusCode::CREATED)
}

pub async fn login_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>, 
) -> Result<StatusCode, (StatusCode, String)> {
    let name = payload.name;
    let raw_pass = payload.pass;

    let salt = SaltString::generate(OsRng);
    let argon2 = Argon2::default();

    let hashed = argon2.hash_password(raw_pass.as_bytes(), &salt).unwrap();

    #[cfg(debug_assertions)]
    println!("LOGIN: {hashed:?}");

    

    Ok(StatusCode::CREATED)
}

