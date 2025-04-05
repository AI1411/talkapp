use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250317_200000_create_table_groups::Groups;
use crate::m20250317_200200_create_table_group_messages::GroupMessages;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupMessageReads::Table)
                    .if_not_exists()
                    .col(pk_auto(GroupMessageReads::Id))
                    .col(integer(GroupMessageReads::GroupId).not_null())
                    .col(integer(GroupMessageReads::MessageId).not_null())
                    .col(integer(GroupMessageReads::UserId).not_null())
                    .col(
                        ColumnDef::new(GroupMessageReads::ReadAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_message_read_group")
                            .from(GroupMessageReads::Table, GroupMessageReads::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_message_read_message")
                            .from(GroupMessageReads::Table, GroupMessageReads::MessageId)
                            .to(GroupMessages::Table, GroupMessages::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_message_read_user")
                            .from(GroupMessageReads::Table, GroupMessageReads::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index to prevent duplicate read records
        manager
            .create_index(
                Index::create()
                    .name("idx_unique_group_message_read")
                    .table(GroupMessageReads::Table)
                    .col(GroupMessageReads::MessageId)
                    .col(GroupMessageReads::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_message_read_group_id")
                    .table(GroupMessageReads::Table)
                    .col(GroupMessageReads::GroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_message_read_message_id")
                    .table(GroupMessageReads::Table)
                    .col(GroupMessageReads::MessageId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_message_read_user_id")
                    .table(GroupMessageReads::Table)
                    .col(GroupMessageReads::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupMessageReads::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum GroupMessageReads {
    Table,
    Id,
    GroupId,
    MessageId,
    UserId,
    ReadAt,
}
