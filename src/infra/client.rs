use sqlx::{PgPool, postgres::PgPoolOptions};

/// DATABASE_URL 環境変数から接続プールを生成します。
pub async fn create_pg_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
}