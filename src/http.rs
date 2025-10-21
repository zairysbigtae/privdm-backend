use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{extract::State, http::StatusCode, Json};
use sqlx::PgPool;
use crate::{jwt::generate_token, user_struct::CreateUser};

pub async fn signup_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>, 
) -> Result<StatusCode, (StatusCode, String)> {
    let name = payload.name;
    let raw_pass = payload.pass;

    let row = sqlx::query!("SELECT name FROM users WHERE name = $1", name)
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

    Ok(StatusCode::CREATED)
}

pub async fn login_handler(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<String, (StatusCode, String)> {
    let name = payload.name;

    let row = sqlx::query!("SELECT name, pass_hash FROM users WHERE name = $1", name)
        .fetch_optional(&pool)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db error".into()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid username".into()))?;

    let parsed_hash = PasswordHash::new(&row.pass_hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Invalid stored hash".into()))?;

    let argon2 = Argon2::default();

    argon2
        .verify_password(payload.pass.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Wrong password".into()))?;

    let key = std::env::var("JWT_KEY").expect("JWT_KEY not set in .env");
    Ok(generate_token(name, key))
}
