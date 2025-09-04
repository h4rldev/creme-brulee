pub use sea_orm_migration::prelude::*;

mod m20250903_152541_create_table_blog_posts;
mod m20250903_154829_create_table_post_stats;
mod m20250903_155308_create_table_users;
mod m20250903_155628_create_table_visit_stats;
mod m20250904_111443_create_table_guestbook;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250903_152541_create_table_blog_posts::Migration),
            Box::new(m20250903_154829_create_table_post_stats::Migration),
            Box::new(m20250903_155308_create_table_users::Migration),
            Box::new(m20250903_155628_create_table_visit_stats::Migration),
            Box::new(m20250904_111443_create_table_guestbook::Migration),
        ]
    }
}
