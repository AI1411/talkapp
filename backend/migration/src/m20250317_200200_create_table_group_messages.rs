use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250317_200000_create_table_groups::Groups;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupMessages::Table)
                    .if_not_exists()
                    .col(pk_auto(GroupMessages::Id))
                    .col(integer(GroupMessages::GroupId).not_null())
                    .col(integer(GroupMessages::SenderId).not_null())
                    .col(string(GroupMessages::Content).not_null())
                    .col(
                        ColumnDef::new(GroupMessages::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(GroupMessages::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(GroupMessages::DeletedAt)
                            .timestamp()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_message_group")
                            .from(GroupMessages::Table, GroupMessages::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_message_sender")
                            .from(GroupMessages::Table, GroupMessages::SenderId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_message_group_id")
                    .table(GroupMessages::Table)
                    .col(GroupMessages::GroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_message_sender_id")
                    .table(GroupMessages::Table)
                    .col(GroupMessages::SenderId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupMessages::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum GroupMessages {
    Table,
    Id,
    GroupId,
    SenderId,
    Content,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
