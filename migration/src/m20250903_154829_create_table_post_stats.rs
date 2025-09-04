use sea_orm_migration::{prelude::*, schema::*, sea_orm::sqlx::types::chrono::Utc};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PostStats::Table)
                    .if_not_exists()
                    .col(pk_auto(PostStats::Id))
                    .col(string(PostStats::PostId).not_null())
                    .col(integer(PostStats::ViewCount).not_null().default(0))
                    .col(integer(PostStats::UniqueVisitors).not_null().default(0))
                    .col(
                        date_time(PostStats::LastViewed)
                            .not_null()
                            .default(Utc::now()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostStats::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum PostStats {
    Table,
    Id,
    PostId,
    ViewCount,
    UniqueVisitors,
    LastViewed,
}
