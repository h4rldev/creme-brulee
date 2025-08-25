pub mod blog_posts;
pub mod post_stats;
pub mod users;
pub mod visit_stats;

// Re-export entities for easier access
pub use blog_posts::{ActiveModel as BlogPostModel, Entity as BlogPosts};
pub use post_stats::{ActiveModel as PostStatModel, Entity as PostStats};
pub use users::{ActiveModel as UserModel, Entity as Users};
pub use visit_stats::{ActiveModel as VisitStatModel, Entity as VisitStats};

// Entity collection for convenience
pub mod entities {
    pub use super::{BlogPostModel, BlogPosts};
    pub use super::{PostStatModel, PostStats};
    pub use super::{UserModel, Users};
    pub use super::{VisitStatModel, VisitStats};
}
