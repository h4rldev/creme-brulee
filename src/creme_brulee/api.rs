use super::{BruleeResult, admin::state::AppState};
use crate::{
    APP_START,
    creme_brulee::database::{
        entities::{GuestbookModel, Guestbooks},
        guestbook,
    },
};
use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{
        HeaderMap, HeaderValue, Response, StatusCode,
        header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE},
    },
    response::{Html, IntoResponse},
};
use chrono::Utc;
use humantime::format_duration;
use mime_guess::mime::APPLICATION_PDF;
use sea_orm::{ActiveModelTrait, EntityTrait, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::{
    fs::{File, read},
    io::AsyncReadExt,
};
use uuid::Uuid;

#[derive(Serialize)]
pub(crate) struct CremeBruleeApiResponse {
    pub status: u16,
    pub message: String,
}

pub(crate) fn creme_brulee_api_err(status: StatusCode, message: &str) -> Response<Body> {
    (
        status,
        Json(CremeBruleeApiResponse {
            status: status.as_u16(),
            message: message.to_string(),
        }),
    )
        .into_response()
}

pub(crate) fn creme_brulee_api_response<T: Serialize>(
    status: StatusCode,
    message: T,
) -> Response<Body> {
    (status, Json(message)).into_response()
}

pub async fn get_api_index() -> impl IntoResponse {
    let current_endpoints = [
        "/cv/en",
        "/cv/sv",
        "/server-info",
        "/uptime",
        "/posts",
        "/posts/slug",
        "/posts/send/69",
    ];

    let wrap_endpoints_with_hyperlinks = current_endpoints
        .iter()
        .map(|endpoint| format!("<li><a href=\"/api{endpoint}\">/api{endpoint}</a></li>"))
        .collect::<Vec<String>>();

    (
        StatusCode::OK,
        Html(format!(
            "<h1>Creme Brulee's shittily hardcoded public api reference</h1>
            <p>Use the links below to access the API endpoints: <br />
            <small>(None of these need authentication, so you can simply access them by going to <code>/api/&lcub;endpoint&rcub;</code> in your browser.)</small>
            </p>
            <p>Note: The API is still under development, so some endpoints may not work as expected.</p>
            <p>Current endpoints:</p>
            <ul>
              {}
            </ul>",
            wrap_endpoints_with_hyperlinks.join("\n")
        )),
    )
}

pub async fn get_cv(headers: HeaderMap, Path(language): Path<String>) -> impl IntoResponse {
    let language_header = match language.to_lowercase().as_str() {
        "en" | "english" => "en",
        "sv" | "se" | "swedish" => "se",
        _ => match headers.get("Language") {
            Some(language) => language.to_str().unwrap_or("Unknown"),
            None => {
                return creme_brulee_api_err(StatusCode::BAD_REQUEST, "Language not supported");
            }
        },
    };

    let file_chosen = match language_header.to_lowercase().as_str() {
        "en" | "english" => "CV - English.pdf",
        "sv" | "se" | "swedish" => "CV - Swedish.pdf",
        _ => "Not available",
    };

    if file_chosen == "Not available" {
        return creme_brulee_api_err(
            StatusCode::NOT_FOUND,
            "CV not available in the requested language",
        );
    }

    let file_path = format!("./assets/{}", file_chosen);
    let contents = match read(&file_path).await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            return creme_brulee_api_err(
                StatusCode::NOT_FOUND,
                "CV file not found, empty, or inaccessible",
            );
        }
    };

    if contents.is_empty() {
        return creme_brulee_api_err(
            StatusCode::NOT_FOUND,
            "CV file not found, empty, or inaccessible",
        );
    }

    let content_length = contents.len();

    if content_length > u64::MAX as usize {
        return creme_brulee_api_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "File size exceeds maximum limit",
        );
    }

    if content_length == 0 {
        return creme_brulee_api_err(
            StatusCode::NOT_FOUND,
            "CV file not found, empty, or inaccessible",
        );
    }

    let content_disposition_hdr =
        match HeaderValue::from_str(&format!("attachment; filename=\"{}\"", file_chosen)) {
            Ok(value) => value,
            Err(_) => {
                return creme_brulee_api_err(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid content disposition header",
                );
            }
        };

    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static(APPLICATION_PDF.as_ref()),
    );
    headers.insert(CONTENT_DISPOSITION, content_disposition_hdr);
    headers.insert(CONTENT_LENGTH, HeaderValue::from(content_length));

    (StatusCode::OK, headers, contents).into_response()
}

#[derive(Serialize)]
struct ServerInfo {
    name: String,
    version: String,
    description: String,
    author: String,
    license: String,
    source: String,
    server_uptime: String,
    system_uptime: String,
}

async fn get_app_uptime() -> BruleeResult<Duration> {
    Ok(Duration::from_secs(
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() - *APP_START,
    ))
}

async fn get_system_uptime() -> BruleeResult<Duration> {
    let mut contents = String::new();
    let mut file = File::open("/proc/uptime").await?;

    file.read_to_string(&mut contents).await?;

    let uptime: f64 = contents
        .split_whitespace()
        .next()
        .ok_or("No data in /proc/uptime")?
        .parse()?;

    if uptime < 0.0 {
        return Err("Uptime cannot be negative".into());
    }

    if uptime > f64::MAX {
        return Err("Uptime exceeds maximum value".into());
    }

    Ok(Duration::from_secs_f64(uptime))
}

pub async fn get_server_info() -> impl IntoResponse {
    let app_uptime = get_app_uptime().await.unwrap_or_else(|e| {
        eprintln!("Error getting app uptime: {}", e);
        Duration::new(0, 0)
    });

    let system_uptime = get_system_uptime().await.unwrap_or_else(|e| {
        eprintln!("Error getting system uptime: {}", e);
        Duration::new(0, 0)
    });

    creme_brulee_api_response(
        StatusCode::OK,
        ServerInfo {
            name: "Creme Brulee".to_string(),
            version: "0.0.1".to_string(),
            description: "A HTTP forwarding web backend".to_string(),
            source: "https://github.com/h4rldev/creme-brulee".to_string(),
            author: "h4rl".to_string(),
            license: "BSD 3-clause License".to_string(),
            server_uptime: format_duration(app_uptime).to_string(),
            system_uptime: format_duration(system_uptime).to_string(),
        },
    )
}

#[derive(Serialize)]
struct UptimeResponse {
    app_uptime: String,
    system_uptime: String,
}

pub async fn get_uptime() -> impl IntoResponse {
    let app_uptime = get_app_uptime().await.unwrap_or_else(|e| {
        eprintln!("Error getting app uptime: {}", e);
        Duration::new(0, 0)
    });

    let system_uptime = get_system_uptime().await.unwrap_or_else(|e| {
        eprintln!("Error getting system uptime: {}", e);
        Duration::new(0, 0)
    });

    creme_brulee_api_response(
        StatusCode::OK,
        UptimeResponse {
            app_uptime: format_duration(app_uptime).to_string(),
            system_uptime: format_duration(system_uptime).to_string(),
        },
    )
}

#[derive(Serialize)]
struct GuestbookResponse {
    id: Uuid,
    title: String,
    content: String,
    author: String,
    created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct GuestbookEntry {
    title: String,
    content: String,
    author: String,
}

pub async fn create_guestbook_entry(
    State(state): State<AppState>,
    Json(payload): Json<GuestbookEntry>,
) -> impl IntoResponse {
    let entry = GuestbookModel {
        id: Set(Uuid::new_v4()),
        title: Set(payload.title),
        content: Set(payload.content),
        author: Set(payload.author),
        created_at: Set(Utc::now()),
    };

    let entry = match entry.insert(&state.db).await {
        Ok(entry) => entry,
        Err(e) => {
            tracing::error!("Failed to create guestbook entry: {}", e);
            return creme_brulee_api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create guestbook entry",
            );
        }
    };

    creme_brulee_api_response(
        StatusCode::CREATED,
        GuestbookResponse {
            id: entry.id,
            title: entry.title,
            content: entry.content,
            author: entry.author,
            created_at: entry.created_at.to_rfc3339(),
        },
    )
}

pub async fn get_guestbook_entries(State(state): State<AppState>) -> impl IntoResponse {
    let entries = match Guestbooks::find()
        .order_by_desc(guestbook::Column::CreatedAt)
        .all(&state.db)
        .await
    {
        Ok(entries) => entries,
        Err(_) => {
            tracing::error!("Failed to fetch guestbook entries");
            vec![]
        }
    };

    let responses = entries
        .into_iter()
        .map(|entry| GuestbookResponse {
            id: entry.id,
            title: entry.title,
            content: entry.content,
            author: entry.author,
            created_at: entry.created_at.to_rfc3339(),
        })
        .collect::<Vec<GuestbookResponse>>();

    creme_brulee_api_response(StatusCode::OK, responses)
}
