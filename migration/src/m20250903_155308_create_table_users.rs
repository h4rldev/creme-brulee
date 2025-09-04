use sea_orm_migration::{prelude::*, schema::*, sea_orm::sqlx::types::chrono::Utc};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_auto(Users::Id))
                    .col(string_uniq(Users::UserId).not_null())
                    .col(string_uniq(Users::Username).not_null())
                    .col(string(Users::PasswordHash).not_null())
                    .col(boolean(Users::IsAdmin).not_null())
                    .col(string(Users::CreatedAt).not_null().default(Utc::now()))
                    .col(string(Users::UpdatedAt).not_null().default(Utc::now()))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_users_username")
                    .table(Users::Table)
                    .col(Users::Username)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    UserId,
    Username,
    PasswordHash,
    IsAdmin,
    CreatedAt,
    UpdatedAt,
}
