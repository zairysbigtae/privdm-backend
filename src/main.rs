use axum::{routing::{any, get, post}, Router};
use privdm_backend::{http::{login_handler, signup_handler}, websocket::ws_handler};
use sqlx::PgPool;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set in .env");
    let pool = PgPool::connect(&url).await.expect("Couldn't connect to database");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/ws", any(ws_handler))
        .route("/signup", post(signup_handler))
        .route("/login", post(login_handler))
        .with_state(pool);

    #[cfg(debug_assertions)] {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:1337").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    #[cfg(not(debug_assertions))] {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:1337").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }
}
