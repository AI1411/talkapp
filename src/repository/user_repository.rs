use crate::domain::entity::users::Model as User;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

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

/// 新規ユーザを作成する
pub async fn create_user(
    pool: &PgPool,
    name: String,
    description: Option<String>,
    age: Option<i32>,
    sex: Option<String>,
) -> Result<User, sqlx::Error> {
    let now = Utc::now().naive_utc();
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (name, description, age, sex, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, description, age, sex, created_at, updated_at, deleted_at
        "#,
        name,
        description,
        age,
        sex,
        now,
        now
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

/// ユーザ情報を更新する
pub async fn update_user(
    pool: &PgPool,
    id: i32,
    name: Option<String>,
    description: Option<String>,
    age: Option<i32>,
    sex: Option<String>,
) -> Result<User, sqlx::Error> {
    let user = get_user_by_id(pool, id).await?;
    let now = Utc::now().naive_utc();

    let updated_user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET
            name = $1,
            description = $2,
            age = $3,
            sex = $4,
            updated_at = $5
        WHERE id = $6
        RETURNING id, name, description, age, sex, created_at, updated_at, deleted_at
        "#,
        name.unwrap_or(user.name),
        description.or(user.description),
        age.or(user.age),
        sex.or(user.sex),
        now,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(updated_user)
}

/// ユーザを論理削除する
pub async fn delete_user(pool: &PgPool, id: i32) -> Result<User, sqlx::Error> {
    let now = Utc::now().naive_utc();
    let user = sqlx::query_as!(
        User,
        r#"
        UPDATE users
        SET deleted_at = $1
        WHERE id = $2
        RETURNING id, name, description, age, sex, created_at, updated_at, deleted_at
        "#,
        Some(now),
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(user)
}

/// ユーザを物理削除する
pub async fn hard_delete_user(pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM users
        WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use sqlx::postgres::PgPoolOptions;
    use std::env;

    async fn setup_test_db() -> PgPool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool")
    }

    // テストユーザーを作成するヘルパー関数
    async fn create_test_user(pool: &PgPool) -> User {
        create_user(
            pool,
            "Test User".to_string(),
            Some("Test Description".to_string()),
            Some(25),
            Some("Male".to_string()),
        )
            .await
            .expect("Failed to create test user")
    }

    // クリーンアップ用のヘルパー関数
    async fn cleanup_user(pool: &PgPool, user_id: i32) {
        hard_delete_user(pool, user_id)
            .await
            .expect("Failed to cleanup test user");
    }

    #[tokio::test]
    async fn test_create_user() {
        let pool = setup_test_db().await;

        let new_user = create_user(
            &pool,
            "Test User".to_string(),
            Some("Test Description".to_string()),
            Some(25),
            Some("Male".to_string()),
        )
            .await
            .expect("Failed to create user");

        assert_eq!(new_user.name, "Test User");
        assert_eq!(new_user.description, Some("Test Description".to_string()));
        assert_eq!(new_user.age, Some(25));
        assert_eq!(new_user.sex, Some("Male".to_string()));
        assert!(new_user.deleted_at.is_none());

        cleanup_user(&pool, new_user.id).await;
    }

    #[tokio::test]
    async fn test_get_user_by_id() {
        let pool = setup_test_db().await;
        let created_user = create_test_user(&pool).await;

        let retrieved_user = get_user_by_id(&pool, created_user.id)
            .await
            .expect("Failed to get user");

        assert_eq!(retrieved_user.id, created_user.id);
        assert_eq!(retrieved_user.name, created_user.name);
        assert_eq!(retrieved_user.description, created_user.description);
        assert_eq!(retrieved_user.age, created_user.age);
        assert_eq!(retrieved_user.sex, created_user.sex);

        cleanup_user(&pool, created_user.id).await;
    }

    #[tokio::test]
    async fn test_list_users() {
        let pool = setup_test_db().await;
        let user1 = create_test_user(&pool).await;
        let user2 = create_user(
            &pool,
            "Another User".to_string(),
            Some("Another Description".to_string()),
            Some(30),
            Some("Female".to_string()),
        )
            .await
            .expect("Failed to create second user");

        let users = list_users(&pool).await.expect("Failed to list users");

        assert!(users.len() >= 2); // データベースに他のデータが存在する可能性があるため
        assert!(users.iter().any(|u| u.id == user1.id));
        assert!(users.iter().any(|u| u.id == user2.id));

        cleanup_user(&pool, user1.id).await;
        cleanup_user(&pool, user2.id).await;
    }

    #[tokio::test]
    async fn test_update_user() {
        let pool = setup_test_db().await;
        let created_user = create_test_user(&pool).await;

        let updated_user = update_user(
            &pool,
            created_user.id,
            Some("Updated Name".to_string()),
            Some("Updated Description".to_string()),
            Some(26),
            Some("Female".to_string()),
        )
            .await
            .expect("Failed to update user");

        assert_eq!(updated_user.id, created_user.id);
        assert_eq!(updated_user.name, "Updated Name");
        assert_eq!(updated_user.description, Some("Updated Description".to_string()));
        assert_eq!(updated_user.age, Some(26));
        assert_eq!(updated_user.sex, Some("Female".to_string()));
        assert!(updated_user.updated_at > created_user.updated_at);

        cleanup_user(&pool, created_user.id).await;
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = setup_test_db().await;
        let created_user = create_test_user(&pool).await;

        // 論理削除のテスト
        let deleted_user = delete_user(&pool, created_user.id)
            .await
            .expect("Failed to delete user");

        assert_eq!(deleted_user.id, created_user.id);
        assert!(deleted_user.deleted_at.is_some());

        // 物理削除のテスト
        hard_delete_user(&pool, created_user.id)
            .await
            .expect("Failed to hard delete user");

        // ユーザーが本当に削除されたか確認
        let result = get_user_by_id(&pool, created_user.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_partial_update() {
        let pool = setup_test_db().await;
        let created_user = create_test_user(&pool).await;

        // 一部のフィールドのみを更新
        let updated_user = update_user(
            &pool,
            created_user.id,
            Some("Updated Name".to_string()),
            None,  // description は更新しない
            None,  // age は更新しない
            None,  // sex は更新しない
        )
            .await
            .expect("Failed to partially update user");

        assert_eq!(updated_user.name, "Updated Name");
        assert_eq!(updated_user.description, created_user.description);
        assert_eq!(updated_user.age, created_user.age);
        assert_eq!(updated_user.sex, created_user.sex);

        cleanup_user(&pool, created_user.id).await;
    }
}
