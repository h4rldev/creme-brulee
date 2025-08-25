use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::{ConnectInfo, Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Set,
};
use serde::Serialize;
use uuid::Uuid;

use super::{
    database::entities::{PostStatModel, PostStats, VisitStatModel, VisitStats},
    state::AppState,
};
use crate::creme_brulee::api::{creme_brulee_api_err, creme_brulee_api_response};

#[derive(Serialize)]
pub struct PostStatsResponse {
    post_id: Uuid,
    view_count: i32,
    unique_visitors: i32,
    last_viewed: String,
}

#[derive(Serialize)]
pub struct VisitStatsResponse {
    total_visits: u64,
    unique_visitors: u64,
    recent_visits: u64,
}

pub async fn track_visit(
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path().to_string();
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Get the real IP address, considering forwarded headers
    let visitor_ip = remote_addr.ip().to_canonical().to_string();

    let visit = VisitStatModel {
        id: Set(0), // Auto-increment
        path: Set(path),
        visitor_ip: Set(visitor_ip),
        user_agent: Set(user_agent),
        timestamp: Set(Utc::now()),
    };

    // Don't block the request if stats tracking fails
    let _ = visit.insert(&state.db).await;

    Ok(next.run(req).await)
}

pub async fn track_post_view(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    ConnectInfo(remote_addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let visitor_ip = remote_addr.ip().to_canonical().to_string();

    let stat = match PostStats::find()
        .filter(super::database::post_stats::Column::PostId.eq(post_id))
        .one(&state.db)
        .await
    {
        Ok(stat) => stat,
        Err(_) => {
            return creme_brulee_api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch post stats",
            );
        }
    };

    match stat {
        Some(stat) => {
            let mut updated_stat: PostStatModel = stat.clone().into();
            updated_stat.view_count = Set(stat.view_count + 1);

            // Check if this IP has viewed before in the last 24 hours
            let recent_visit = match VisitStats::find()
                .filter(
                    super::database::visit_stats::Column::Path.eq(format!("/posts/{}", post_id)),
                )
                .filter(super::database::visit_stats::Column::VisitorIp.eq(visitor_ip))
                .filter(
                    super::database::visit_stats::Column::Timestamp
                        .gt(Utc::now() - chrono::Duration::hours(24)),
                )
                .one(&state.db)
                .await
            {
                Ok(recent_visit) => recent_visit,
                Err(_) => {
                    return creme_brulee_api_err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to fetch recent visit",
                    );
                }
            };

            if recent_visit.is_none() {
                updated_stat.unique_visitors = Set(stat.unique_visitors + 1);
            }

            updated_stat.last_viewed = Set(Utc::now());
            match updated_stat.update(&state.db).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Failed to update post stat: {}", e);
                    return creme_brulee_api_err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to update post stat",
                    );
                }
            }
        }
        None => {
            let new_stat = PostStatModel {
                id: Set(0), // Auto-increment
                post_id: Set(post_id),
                view_count: Set(1),
                unique_visitors: Set(1),
                last_viewed: Set(Utc::now()),
            };

            match new_stat.insert(&state.db).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Failed to insert post stat: {}", e);
                    return creme_brulee_api_err(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to insert post stat",
                    );
                }
            }
        }
    }

    creme_brulee_api_response(StatusCode::OK, "Post viewed")
}

pub async fn get_post_stats(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> impl IntoResponse {
    let stat = match PostStats::find()
        .filter(super::database::post_stats::Column::PostId.eq(post_id))
        .one(&state.db)
        .await
    {
        Ok(Some(stat)) => stat,
        Ok(None) => {
            return creme_brulee_api_response(StatusCode::NOT_FOUND, "Post stats not found");
        }
        Err(_) => {
            return creme_brulee_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to fetch post stats",
            );
        }
    };

    creme_brulee_api_response(
        StatusCode::OK,
        PostStatsResponse {
            post_id: stat.post_id,
            view_count: stat.view_count,
            unique_visitors: stat.unique_visitors,
            last_viewed: stat.last_viewed.to_rfc3339(),
        },
    )
}

pub async fn get_visit_stats(State(state): State<AppState>) -> impl IntoResponse {
    let total_visits = match VisitStats::find().count(&state.db).await {
        Ok(count) => count,
        Err(_) => {
            return creme_brulee_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to count total visits",
            );
        }
    };

    let unique_visitors = match VisitStats::find()
        .select_only()
        .column(super::database::visit_stats::Column::VisitorIp)
        .distinct()
        .count(&state.db)
        .await
    {
        Ok(count) => count,
        Err(_) => {
            return creme_brulee_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to count unique visitors",
            );
        }
    };

    let recent_visits = match VisitStats::find()
        .filter(
            super::database::visit_stats::Column::Timestamp
                .gt(Utc::now() - chrono::Duration::hours(24)),
        )
        .count(&state.db)
        .await
    {
        Ok(count) => count,
        Err(_) => {
            return creme_brulee_api_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to count recent visits",
            );
        }
    };

    creme_brulee_api_response(
        StatusCode::OK,
        VisitStatsResponse {
            total_visits,
            unique_visitors,
            recent_visits,
        },
    )
}
