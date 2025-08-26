use axum_extra::extract::cookie::Key;
use dotenvy::dotenv;
use sea_orm::DatabaseConnection;
use tracing::info;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub key: Key,
}

impl AppState {
    pub async fn new() -> Self {
        dotenv().ok();

        let key = if std::env::var("COOKIE_SECRET").is_err() {
            let _key = Key::generate();
            info!(
                "COOKIE_SECRET not set, generated one: {}",
                _key.master()
                    .iter()
                    .map(|c| { c.to_ascii_lowercase().to_string() })
                    .collect::<String>()
            );
            _key
        } else {
            let _key = Key::from(
                std::env::var("COOKIE_SECRET")
                    .expect("COOKIE_SECRET must be set")
                    .as_bytes(),
            );
            info!(
                "COOKIE_SECRET set to: {}",
                _key.master()
                    .iter()
                    .map(|c| { c.to_ascii_lowercase().to_string() })
                    .collect::<String>()
            );
            _key
        };

        let db = sea_orm::Database::connect(
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        )
        .await
        .expect("Failed to connect to database");

        Self { db, key }
    }
}
