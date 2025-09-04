use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Guestbook::Table)
                    .if_not_exists()
                    .col(pk_auto(Guestbook::Id))
                    .col(string(Guestbook::Title))
                    .col(string(Guestbook::Content))
                    .col(string(Guestbook::Author))
                    .col(date_time(Guestbook::CreatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Guestbook::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Guestbook {
    Table,
    Id,
    Title,
    Content,
    Author,
    CreatedAt,
}
