use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Reactions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Reactions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Reactions::UserId).integer().not_null())
                    .col(ColumnDef::new(Reactions::MessageId).integer().not_null())
                    .col(ColumnDef::new(Reactions::ReactionTypeId).integer().not_null())
                    .col(
                        ColumnDef::new(Reactions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Reactions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Reactions::DeletedAt).timestamp_with_time_zone())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reactions_user_id")
                            .from(Reactions::Table, Reactions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reactions_message_id")
                            .from(Reactions::Table, Reactions::MessageId)
                            .to(Messages::Table, Messages::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_reactions_reaction_type_id")
                            .from(Reactions::Table, Reactions::ReactionTypeId)
                            .to(ReactionTypes::Table, ReactionTypes::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // ユーザーとメッセージとリアクションタイプの組み合わせに対してユニーク制約を追加
        // 同じユーザーが同じメッセージに同じリアクションを複数回付けられないようにする
        manager
            .create_index(
                Index::create()
                    .name("idx_reactions_unique")
                    .table(Reactions::Table)
                    .col(Reactions::UserId)
                    .col(Reactions::MessageId)
                    .col(Reactions::ReactionTypeId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // メッセージIDに対するインデックスを追加（メッセージごとのリアクション取得を高速化）
        manager
            .create_index(
                Index::create()
                    .name("idx_reactions_message_id")
                    .table(Reactions::Table)
                    .col(Reactions::MessageId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Reactions::Table).to_owned())
            .await
    }
}

/// リアクションを管理するテーブル
#[derive(Iden)]
enum Reactions {
    Table,
    Id,
    UserId,
    MessageId,
    ReactionTypeId,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}

/// 外部キー用のユーザーテーブル参照
#[derive(Iden)]
enum Users {
    Table,
    Id,
}

/// 外部キー用のメッセージテーブル参照
#[derive(Iden)]
enum Messages {
    Table,
    Id,
}

/// 外部キー用のリアクションタイプテーブル参照
#[derive(Iden)]
enum ReactionTypes {
    Table,
    Id,
}
