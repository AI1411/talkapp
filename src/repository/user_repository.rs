use sqlx::PgPool;
use crate::domain::entity::users::Model as User;

/// ユーザIDでユーザを取得する
pub async fn get_user_by_id(pool: &PgPool, id: i32) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, description, age, sex, created_at, updated_at, deleted_at
        FROM users
        WHERE id = $1
        "#,
        id
    )
        .fetch_one(pool)
        .await?;
    Ok(user)
}

/// ユーザ一覧を取得する
pub async fn list_users(pool: &PgPool) -> Result<Vec<User>, sqlx::Error> {
    let users = sqlx::query_as!(
        User,
        r#"
        SELECT id, name, description, age, sex, created_at, updated_at, deleted_at
        FROM users
        ORDER BY id
        "#
    )
        .fetch_all(pool)
        .await?;
    Ok(users)
}