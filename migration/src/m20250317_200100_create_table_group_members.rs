use sea_orm::sea_query::Expr;
use sea_orm_migration::prelude::*;

use crate::m20250317_200000_create_table_groups::Groups;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(GroupMembers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(GroupMembers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(GroupMembers::GroupId).integer().not_null())
                    .col(ColumnDef::new(GroupMembers::UserId).integer().not_null())
                    .col(ColumnDef::new(GroupMembers::Role).string().not_null().default("member"))
                    .col(
                        ColumnDef::new(GroupMembers::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(GroupMembers::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(ColumnDef::new(GroupMembers::DeletedAt).timestamp().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_member_group")
                            .from(GroupMembers::Table, GroupMembers::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_group_member_user")
                            .from(GroupMembers::Table, GroupMembers::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique index to prevent duplicate memberships
        manager
            .create_index(
                Index::create()
                    .name("idx_unique_group_member")
                    .table(GroupMembers::Table)
                    .col(GroupMembers::GroupId)
                    .col(GroupMembers::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_id")
                    .table(GroupMembers::Table)
                    .col(GroupMembers::GroupId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_group_members_user_id")
                    .table(GroupMembers::Table)
                    .col(GroupMembers::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(GroupMembers::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum GroupMembers {
    Table,
    Id,
    GroupId,
    UserId,
    Role,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
