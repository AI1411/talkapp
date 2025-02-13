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

    #[tokio::test]
    async fn test_crud_operations() {
        let pool = setup_test_db().await;

        // 1. Create user
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
        assert_eq!(new_user.age, Some(25));
        println!("Created user: {:?}", new_user);

        // 2. Get user
        let retrieved_user = get_user_by_id(&pool, new_user.id)
            .await
            .expect("Failed to get user");

        assert_eq!(retrieved_user.id, new_user.id);
        assert_eq!(retrieved_user.name, new_user.name);
        println!("Retrieved user: {:?}", retrieved_user);

        // 3. Update user
        let updated_user = update_user(
            &pool,
            new_user.id,
            Some("Updated User".to_string()),
            Some("Updated Description".to_string()),
            Some(26),
            None,
        )
        .await
        .expect("Failed to update user");

        assert_eq!(updated_user.name, "Updated User");
        assert_eq!(updated_user.age, Some(26));
        println!("Updated user: {:?}", updated_user);

        // 4. List users
        let users = list_users(&pool).await.expect("Failed to list users");

        assert!(!users.is_empty());
        println!("Listed {} users", users.len());

        // 5. Delete user
        let deleted_user = delete_user(&pool, new_user.id)
            .await
            .expect("Failed to delete user");

        assert!(deleted_user.deleted_at.is_some());
        println!("Deleted user: {:?}", deleted_user);

        // Cleanup: Hard delete the test user
        hard_delete_user(&pool, new_user.id)
            .await
            .expect("Failed to hard delete user");
    }

    #[tokio::test]
    async fn test_bulk_insert() {
        let pool = setup_test_db().await;

        // バルクインサートのテストデータ
        let test_users = vec![
            ("Alice Johnson", "Engineer", 28, "Female"),
            ("Bob Smith", "Designer", 32, "Male"),
            ("Carol Williams", "Manager", 35, "Female"),
            ("David Brown", "Developer", 25, "Male"),
            ("Eve Davis", "Architect", 30, "Female"),
        ];

        // テストユーザーを作成
        for (name, desc, age, sex) in test_users {
            let user = create_user(
                &pool,
                name.to_string(),
                Some(desc.to_string()),
                Some(age),
                Some(sex.to_string()),
            )
            .await
            .expect("Failed to create test user");

            println!("Created test user: {:?}", user);
        }

        // 全ユーザーを取得して確認
        let all_users = list_users(&pool).await.expect("Failed to list users");

        println!("Total users in database: {}", all_users.len());
        for user in &all_users {
            println!("User: {:?}", user);
        }

        for user in all_users {
            hard_delete_user(&pool, user.id)
                .await
                .expect("Failed to cleanup test user");
        }
    }
}
