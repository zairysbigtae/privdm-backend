use axum::{routing::{any, get}, Router};
use privdm_backend::websocket::ws_handler;
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
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:1337").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
