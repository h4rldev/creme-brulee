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
                    .table(PostStats::Table)
                    .if_not_exists()
                    .col(pk_auto(PostStats::Id))
                    .col(uuid(PostStats::PostId))
                    .col(integer(PostStats::ViewCount))
                    .col(integer(PostStats::UniqueVisitors))
                    .col(date_time(PostStats::LastViewed))
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
