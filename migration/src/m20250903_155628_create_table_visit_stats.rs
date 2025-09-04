use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(VisitStats::Table)
                    .if_not_exists()
                    .col(pk_auto(VisitStats::Id))
                    .col(string(VisitStats::Path))
                    .col(string(VisitStats::VisitorIp))
                    .col(string(VisitStats::UserAgent))
                    .col(date_time(VisitStats::Timestamp))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("visit_stats_visitor_ip_idx")
                    .table(VisitStats::Table)
                    .col(VisitStats::VisitorIp)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("visit_stats_path_idx")
                    .table(VisitStats::Table)
                    .col(VisitStats::Path)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("visit_stats_timestamp_idx")
                    .table(VisitStats::Table)
                    .col(VisitStats::Timestamp)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("visit_stats_timestamp_idx")
                    .table(VisitStats::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("visit_stats_path_idx")
                    .table(VisitStats::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("visit_stats_visitor_ip_idx")
                    .table(VisitStats::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(VisitStats::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum VisitStats {
    Table,
    Id,
    Path,
    VisitorIp,
    UserAgent,
    Timestamp,
}
