use sea_orm_migration::{prelude::*, schema::*, sea_orm::sqlx::types::chrono::Utc};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BlogPosts::Table)
                    .if_not_exists()
                    .col(pk_auto(BlogPosts::Id))
                    .col(uuid(BlogPosts::PostId).not_null())
                    .col(string(BlogPosts::Title).not_null())
                    .col(string(BlogPosts::Slug).not_null())
                    .col(string(BlogPosts::Content).not_null())
                    .col(boolean(BlogPosts::Published).not_null())
                    .col(
                        string(BlogPosts::CreatedAt)
                            .not_null()
                            .default(Utc::now().to_rfc3339()),
                    )
                    .col(
                        string(BlogPosts::UpdatedAt)
                            .not_null()
                            .default(Utc::now().to_rfc3339()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("blog_posts_slug_idx")
                    .table(BlogPosts::Table)
                    .col(BlogPosts::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("blog_posts_slug_idx")
                    .table(BlogPosts::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(BlogPosts::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum BlogPosts {
    Table,
    Id,
    PostId,
    Title,
    Slug,
    Content,
    Published,
    CreatedAt,
    UpdatedAt,
}
