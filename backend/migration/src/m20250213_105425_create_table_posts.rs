use crate::m20250213_100210_create_table_users::Users;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(pk_auto(Post::Id))
                    .col(string(Post::Body).not_null())
                    .col(integer(Post::UserId).not_null())
                    .col(
                        string(Post::CreatedAt)
                            .not_null()
                            .default("CURRENT_TIMESTAMP"),
                    )
                    .to_owned(),
            )
            .await
            .expect("create table posts");

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_user_id")
                    .from(Post::Table, Post::UserId)
                    .to(Users::Table, Users::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
            .expect("create foreign key fk_user_id");

        manager
            .create_index(
                Index::create()
                    .name("idx_user_id")
                    .table(Post::Table)
                    .col(Post::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    Body,
    UserId,
    CreatedAt,
}
