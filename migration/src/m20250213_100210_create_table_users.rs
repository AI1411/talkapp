use sea_orm::sea_query::Expr;
use sea_orm_migration::prelude::*;

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
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(ColumnDef::new(Users::Description).string().null())
                    .col(ColumnDef::new(Users::Age).integer().null())
                    .col(ColumnDef::new(Users::Sex).string().null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::cust("CURRENT_TIMESTAMP")),
                    )
                    .col(ColumnDef::new(Users::DeletedAt).timestamp().null())
                    .to_owned(),
            )
            .await
            .expect("create table users");

        manager
            .create_index(
                Index::create()
                    .name("idx_name")
                    .table(Users::Table)
                    .col(Users::Name)
                    .to_owned(),
            )
            .await
            .expect("create index idx_name");

        manager
            .create_index(
                Index::create()
                    .name("idx_age")
                    .table(Users::Table)
                    .col(Users::Age)
                    .to_owned(),
            )
            .await
            .expect("create index idx_age");

        manager
            .create_index(
                Index::create()
                    .name("idx_sex")
                    .table(Users::Table)
                    .col(Users::Sex)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    Name,
    Description,
    Sex,
    Age,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
}
