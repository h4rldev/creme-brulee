use axum_extra::extract::cookie::Key;
use dotenvy::dotenv;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub key: Key,
}

impl AppState {
    pub async fn new() -> Self {
        dotenv().ok();

        let key = if std::env::var("COOKIE_SECRET").is_err() {
            Key::generate()
        } else {
            Key::from(
                std::env::var("COOKIE_SECRET")
                    .expect("COOKIE_SECRET must be set")
                    .as_bytes(),
            )
        };

        let db = sea_orm::Database::connect(std::env::var("DATABASE_URL").unwrap())
            .await
            .expect("Failed to connect to database");

        Self { db, key }
    }
}
