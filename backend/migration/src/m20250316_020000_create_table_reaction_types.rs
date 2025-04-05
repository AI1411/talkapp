use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ReactionTypes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReactionTypes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ReactionTypes::Name).string().not_null())
                    .col(ColumnDef::new(ReactionTypes::Emoji).string().not_null())
                    .col(
                        ColumnDef::new(ReactionTypes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ReactionTypes::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 初期データの挿入
        manager
            .exec_stmt(
                Query::insert()
                    .into_table(ReactionTypes::Table)
                    .columns([ReactionTypes::Name, ReactionTypes::Emoji])
                    .values_panic(["いいね".into(), "👍".into()])
                    .values_panic(["わかる".into(), "🙂".into()])
                    .values_panic(["応援してる".into(), "🎉".into()])
                    .values_panic(["おつかれさま".into(), "🙏".into()])
                    .values_panic(["たしかに".into(), "🤔".into()])
                    .values_panic(["すごい".into(), "🔥".into()])
                    .values_panic(["笑った".into(), "😂".into()])
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ReactionTypes::Table).to_owned())
            .await
    }
}

/// 反応の種類を管理するテーブル
#[derive(Iden)]
enum ReactionTypes {
    Table,
    Id,
    Name,
    Emoji,
    CreatedAt,
    UpdatedAt,
}
