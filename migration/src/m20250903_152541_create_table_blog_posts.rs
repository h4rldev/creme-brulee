use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(BlogPosts::Table)
                    .if_not_exists()
                    .col(pk_uuid(BlogPosts::Id))
                    .col(string(BlogPosts::Title))
                    .col(string(BlogPosts::Slug))
                    .col(string(BlogPosts::Content))
                    .col(boolean(BlogPosts::Published))
                    .col(date_time(BlogPosts::CreatedAt))
                    .col(date_time(BlogPosts::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("blog_posts_slug_idx")
                    .table(BlogPosts::Table)
                    .col(BlogPosts::Slug)
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
    Title,
    Slug,
    Content,
    Published,
    CreatedAt,
    UpdatedAt,
}
