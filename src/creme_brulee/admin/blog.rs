use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::creme_brulee::{
    admin,
    api::{creme_brulee_api_err, creme_brulee_api_response},
};

use super::{
    database::entities::{BlogPostModel, BlogPosts},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreatePostPayload {
    title: String,
    content: String,
    published: bool,
}

#[derive(Debug, Serialize)]
pub struct BlogPostResponse {
    id: Uuid,
    title: String,
    slug: String,
    content: String,
    published: bool,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostPayload {
    title: Option<String>,
    content: Option<String>,
    published: Option<bool>,
}

// Public endpoints
pub async fn get_published_posts(State(state): State<AppState>) -> impl IntoResponse {
    let posts = match BlogPosts::find()
        .filter(super::database::blog_posts::Column::Published.eq(true))
        .order_by_desc(super::database::blog_posts::Column::CreatedAt)
        .all(&state.db)
        .await
    {
        Ok(blog_posts) => blog_posts,
        Err(_) => {
            tracing::error!("Failed to fetch published blog posts");
            vec![]
        }
    };

    let responses = posts
        .into_iter()
        .map(|post| BlogPostResponse {
            id: post.id,
            title: post.title,
            slug: post.slug,
            content: post.content,
            published: post.published,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        })
        .collect::<Vec<_>>();

    creme_brulee_api_response(StatusCode::OK, responses)
}

pub async fn get_published_post_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let post = match BlogPosts::find()
        .filter(super::database::blog_posts::Column::Slug.eq(&slug))
        .filter(super::database::blog_posts::Column::Published.eq(true))
        .one(&state.db)
        .await
    {
        Ok(post) => post,
        Err(_) => {
            tracing::error!("Failed to fetch blog post by slug: {}", &slug);
            None
        }
    };

    let post = post.unwrap_or_else(|| {
        tracing::warn!("Blog post with slug '{}' not found", slug);
        admin::database::blog_posts::Model {
            id: Uuid::new_v4(),
            title: "Not Found".to_string(),
            slug: slug.clone(),
            content: "The requested blog post does not exist.".to_string(),
            published: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    });

    creme_brulee_api_response(
        StatusCode::OK,
        BlogPostResponse {
            id: post.id,
            title: post.title,
            slug: post.slug,
            content: post.content,
            published: post.published,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        },
    )
}

// Admin endpoints
pub async fn get_all_posts(State(state): State<AppState>) -> impl IntoResponse {
    let posts = match BlogPosts::find()
        .order_by_desc(super::database::blog_posts::Column::CreatedAt)
        .all(&state.db)
        .await
    {
        Ok(posts) => posts,
        Err(_) => {
            tracing::error!("Failed to fetch all blog posts");
            vec![]
        }
    };

    let responses = posts
        .into_iter()
        .map(|post| BlogPostResponse {
            id: post.id,
            title: post.title,
            slug: post.slug,
            content: post.content,
            published: post.published,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        })
        .collect::<Vec<BlogPostResponse>>();

    creme_brulee_api_response(StatusCode::OK, responses);
}

pub async fn get_post_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> impl IntoResponse {
    let post = match BlogPosts::find()
        .filter(super::database::blog_posts::Column::Slug.eq(slug.clone()))
        .one(&state.db)
        .await
    {
        Ok(post) => post,
        Err(_) => {
            tracing::error!("Failed to fetch blog post by slug: {}", &slug);
            None
        }
    };

    let post = post.unwrap_or_else(|| {
        tracing::warn!("Blog post with slug '{}' not found", &slug);
        admin::database::blog_posts::Model {
            id: Uuid::new_v4(),
            title: "Not Found".to_string(),
            slug: slug.clone(),
            content: "The requested blog post does not exist.".to_string(),
            published: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    });

    creme_brulee_api_response(
        StatusCode::OK,
        BlogPostResponse {
            id: post.id,
            title: post.title,
            slug: post.slug,
            content: post.content,
            published: post.published,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        },
    )
}

pub async fn create_post(
    State(state): State<AppState>,
    Json(payload): Json<CreatePostPayload>,
) -> impl IntoResponse {
    let slug = slugify(&payload.title);

    let post = BlogPostModel {
        id: Set(Uuid::new_v4()),
        title: Set(payload.title),
        slug: Set(slug),
        content: Set(payload.content),
        published: Set(payload.published),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let post = match post.insert(&state.db).await {
        Ok(post) => post,
        Err(e) => {
            tracing::error!("Failed to create blog post");
            return creme_brulee_api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to create blog post: {}", e),
            );
        }
    };

    creme_brulee_api_response(
        StatusCode::CREATED,
        BlogPostResponse {
            id: post.id,
            title: post.title,
            slug: post.slug,
            content: post.content,
            published: post.published,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        },
    )
}

pub async fn update_post(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdatePostPayload>,
) -> impl IntoResponse {
    let post = match BlogPosts::find_by_id(id).one(&state.db).await {
        Ok(post) => post,
        Err(_) => {
            tracing::error!("Failed to fetch blog post by id: {}", id);
            None
        }
    };

    let mut post: BlogPostModel = post.expect("Blog post not found").into();

    if let Some(title) = payload.title {
        post.title = Set(title.clone());
        post.slug = Set(slugify(&title));
    }

    if let Some(content) = payload.content {
        post.content = Set(content);
    }

    if let Some(published) = payload.published {
        post.published = Set(published);
    }

    post.updated_at = Set(Utc::now());

    let post = match post.update(&state.db).await {
        Ok(post) => post,
        Err(e) => {
            tracing::error!("Failed to update blog post");
            return creme_brulee_api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to update blog post: {}", e),
            );
        }
    };

    creme_brulee_api_response(
        StatusCode::OK,
        BlogPostResponse {
            id: post.id,
            title: post.title,
            slug: post.slug,
            content: post.content,
            published: post.published,
            created_at: post.created_at.to_rfc3339(),
            updated_at: post.updated_at.to_rfc3339(),
        },
    )
}

pub async fn delete_post(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let post = match BlogPosts::find_by_id(id).one(&state.db).await {
        Ok(post) => post,
        Err(_) => {
            tracing::error!("Failed to fetch blog post by id: {}", id);
            None
        }
    };

    let post = match post {
        Some(post) => post,
        None => {
            tracing::error!("Failed to fetch blog post by id: {}", id);
            return creme_brulee_api_err(StatusCode::NOT_FOUND, "Blog post not found");
        }
    };

    match BlogPosts::delete_by_id(post.id).exec(&state.db).await {
        Ok(_) => creme_brulee_api_response(
            StatusCode::NO_CONTENT,
            format!("Blog post {} deleted", &post.id),
        ),
        Err(e) => {
            tracing::error!("Failed to delete blog post: {}", e);
            creme_brulee_api_err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete blog post",
            )
        }
    }
}

fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>()
        .replace("--", "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("This is a Test!"), "this-is-a-test");
        assert_eq!(slugify("Multiple   Spaces"), "multiple-spaces");
        assert_eq!(slugify("Special@#$Characters"), "specialcharacters");
        assert_eq!(slugify("with-dash-already"), "with-dash-already");
        assert_eq!(slugify("Mixed   CASE"), "mixed-case");
    }
}
