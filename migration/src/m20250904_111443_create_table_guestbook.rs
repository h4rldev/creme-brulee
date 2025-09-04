use sea_orm_migration::{prelude::*, schema::*, sea_orm::sqlx::types::chrono::Utc};

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
                    .col(string(Guestbook::Title).not_null())
                    .col(string(Guestbook::Content).not_null())
                    .col(string(Guestbook::Author).not_null())
                    .col(
                        string(Guestbook::CreatedAt)
                            .not_null()
                            .default(Utc::now().to_rfc3339()),
                    )
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
