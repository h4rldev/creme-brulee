use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, PrivateCookieJar, SameSite};
use chrono::{Duration, Utc};
use dotenvy::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use once_cell::sync::Lazy;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    super::api::{creme_brulee_api_err, creme_brulee_api_response},
    database::entities::{UserModel, Users},
    state::AppState,
};

static JWT_SECRET: Lazy<String> = Lazy::new(|| {
    dotenv().ok();
    std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String, // User ID
    exp: i64,    // Expiration timestamp
    iat: i64,    // Issued at timestamp
    admin: bool, // Is admin
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct InitialSetupPayload {
    username: String,
    password: String,
    setup_key: String,
}

/* POST /admin/auth/login
 *
 * {
 *   "username": "",
 *   "password": ""
 * }
 */

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> impl IntoResponse {
    let jar = PrivateCookieJar::new(state.key.clone());

    let user = match Users::find()
        .filter(super::database::users::Column::Username.eq(payload.username))
        .one(&state.db)
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return creme_brulee_api_err(StatusCode::UNAUTHORIZED, "Invalid username or password");
        }
        Err(_) => {
            return creme_brulee_api_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user");
        }
    };

    let parsed_hash = match PasswordHash::new(&user.password_hash) {
        Ok(hash) => hash,
        Err(_) => {
            return creme_brulee_api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to parse password hash",
            );
        }
    };

    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return creme_brulee_api_err(StatusCode::UNAUTHORIZED, "Invalid username or password");
    }

    let now = Utc::now();
    let exp = now + Duration::hours(24);

    let claims = Claims {
        sub: user.id.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        admin: user.is_admin,
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_bytes()),
    ) {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, jar).into_response(),
    };

    let cookie = Cookie::build(("auth_token", token))
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .build();

    (StatusCode::OK, jar.add(cookie)).into_response()
}

/* POST /admin/auth/setup
 *
 * {
 *   "username": "",
 *   "password": "",
 *   "setup_key": ""
 * }
 */

pub async fn initial_setup(
    State(state): State<AppState>,
    Json(payload): axum::Json<InitialSetupPayload>,
) -> impl IntoResponse {
    // Check if any admin user exists
    dotenv().ok();

    let admin_exists = match Users::find()
        .filter(super::database::users::Column::IsAdmin.eq(true))
        .one(&state.db)
        .await
    {
        Ok(exists) => exists.is_some(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if admin_exists {
        return StatusCode::FORBIDDEN.into_response();
    }

    // Verify setup key
    let setup_key = match std::env::var("ADMIN_SETUP_KEY") {
        Ok(key) => key,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if payload.setup_key != setup_key {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = match argon2.hash_password(payload.password.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let user = UserModel {
        id: Set(Uuid::new_v4()),
        username: Set(payload.username),
        password_hash: Set(password_hash),
        is_admin: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    match user.insert(&state.db).await {
        Ok(_) => StatusCode::CREATED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/* Helper function to verify if the user is an admin */

async fn verify_admin(jar: &PrivateCookieJar, db: &DatabaseConnection) -> impl IntoResponse {
    let token = match jar.get("auth_token") {
        Some(token) => token.value().to_string(),
        None => return creme_brulee_api_err(StatusCode::UNAUTHORIZED, "No auth token"),
    };

    let token_data = match decode::<Claims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    ) {
        Ok(token_data) => token_data,
        Err(_) => return creme_brulee_api_err(StatusCode::UNAUTHORIZED, "Invalid token"),
    };

    let user_id = match Uuid::parse_str(&token_data.claims.sub) {
        Ok(user_id) => user_id,
        Err(_) => return creme_brulee_api_err(StatusCode::UNAUTHORIZED, "Invalid token"),
    };

    // A little extra wall for security, checking both claims and database
    if !token_data.claims.admin {
        return creme_brulee_api_err(StatusCode::FORBIDDEN, "User is not an admin");
    }

    let user = match Users::find_by_id(user_id).one(db).await {
        Ok(user) => user,
        Err(_) => {
            return creme_brulee_api_err(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user");
        }
    };

    let user = match user {
        Some(user) => user,
        None => {
            return creme_brulee_api_err(StatusCode::UNAUTHORIZED, "Invalid token");
        }
    };

    if !user.is_admin {
        return creme_brulee_api_err(StatusCode::FORBIDDEN, "User is not an admin");
    }

    creme_brulee_api_response(StatusCode::OK, "Admin verified")
}

/* Middleware to check if the user is an admin */

pub async fn require_admin(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let jar = request
        .extensions()
        .get::<PrivateCookieJar>()
        .cloned()
        .unwrap_or_else(|| PrivateCookieJar::new(state.key.clone()));

    verify_admin(&jar, &state.db).await;
    Ok(next.run(request).await)
}

/* GET /admin/auth/logout */

pub async fn logout(State(state): State<AppState>, request: Request<Body>) -> impl IntoResponse {
    let jar = request
        .extensions()
        .get::<PrivateCookieJar>()
        .cloned()
        .unwrap_or_else(|| PrivateCookieJar::new(state.key.clone()));

    let stat = jar.remove(Cookie::from("auth_token"));

    (StatusCode::OK, stat).into_response()
}
