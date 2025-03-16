use crate::domain::entity::{reaction_types, reactions};
use crate::domain::repository::reaction::ReactionRepository;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, DbBackend, DbErr, Statement};
use sea_orm::QueryOrder;

pub struct PgReactionRepository {
    db: DatabaseConnection,
}

impl PgReactionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ReactionRepository for PgReactionRepository {
    async fn add_reaction(
        &self,
        user_id: i32,
        message_id: i32,
        reaction_type_id: i32,
    ) -> Result<reactions::Model, DbErr> {
        // 直接SQLでINSERTを実行し、IDを取得
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO reactions (user_id, message_id, reaction_type_id, created_at, updated_at) 
             VALUES ($1, $2, $3, NOW(), NOW()) RETURNING id",
            vec![user_id.into(), message_id.into(), reaction_type_id.into()],
        );
        let result = self.db.query_one(stmt).await?;
        let id = result.unwrap().try_get::<i32>("", "id").unwrap();

        // IDでエンティティを取得
        let reaction = reactions::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("リアクションの挿入に失敗しました".into()))?;
        Ok(reaction)
    }

    async fn remove_reaction(
        &self,
        user_id: i32,
        message_id: i32,
        reaction_type_id: Option<i32>,
    ) -> Result<i32, DbErr> {
        let now = Utc::now().naive_utc();
        let mut query = reactions::Entity::update_many()
            .filter(reactions::Column::UserId.eq(user_id))
            .filter(reactions::Column::MessageId.eq(message_id))
            .filter(reactions::Column::DeletedAt.is_null());

        if let Some(type_id) = reaction_type_id {
            query = query.filter(reactions::Column::ReactionTypeId.eq(type_id));
        }

        let result = query
            .col_expr(reactions::Column::DeletedAt, Expr::value(Some(now)))
            .col_expr(reactions::Column::UpdatedAt, Expr::value(now))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected as i32)
    }

    async fn get_reactions_for_message(
        &self,
        message_id: i32,
    ) -> Result<Vec<reactions::Model>, DbErr> {
        let reactions = reactions::Entity::find()
            .filter(reactions::Column::MessageId.eq(message_id))
            .filter(reactions::Column::DeletedAt.is_null())
            .order_by_asc(reactions::Column::CreatedAt)
            .all(&self.db)
            .await?;
        Ok(reactions)
    }

    async fn count_reactions_by_type(
        &self,
        message_id: i32,
    ) -> Result<Vec<(reaction_types::Model, i64)>, DbErr> {
        // 直接SQLで集計を実行
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT reaction_type_id, COUNT(*) as count 
             FROM reactions 
             WHERE message_id = $1 AND deleted_at IS NULL 
             GROUP BY reaction_type_id",
            vec![message_id.into()],
        );

        let count_results = self.db.query_all(stmt).await?;
        let mut result: Vec<(reaction_types::Model, i64)> = Vec::new();

        for row in count_results {
            let type_id: i32 = row.try_get("", "reaction_type_id").unwrap();
            let count: i64 = row.try_get("", "count").unwrap();

            if let Some(reaction_type) = reaction_types::Entity::find_by_id(type_id)
                .one(&self.db)
                .await?
            {
                result.push((reaction_type, count));
            }
        }

        Ok(result)
    }

    async fn list_reaction_types(&self) -> Result<Vec<reaction_types::Model>, DbErr> {
        let types = reaction_types::Entity::find()
            .order_by_asc(reaction_types::Column::Id)
            .all(&self.db)
            .await?;
        Ok(types)
    }

    async fn get_reaction_type(&self, id: i32) -> Result<Option<reaction_types::Model>, DbErr> {
        let reaction_type = reaction_types::Entity::find_by_id(id).one(&self.db).await?;
        Ok(reaction_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use dotenv::dotenv;
    use sea_orm::{Database, DatabaseConnection};
    use std::env;

    /// テスト用のDB接続を取得します
    async fn setup_db() -> DatabaseConnection {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Database::connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    /// Generates a unique test ID to avoid conflicts between tests
    fn generate_test_id() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_nanos();
        format!("test_{}", now)
    }

    /// テスト用のデータをクリーンアップします
    async fn cleanup_test_data(db: &DatabaseConnection) -> Result<(), DbErr> {
        // テストで作成したリアクションを物理的に削除
        let stmt1 = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM reactions WHERE user_id IN (1, 2)",
            vec![],
        );
        db.execute(stmt1).await?;

        // テストで作成したメッセージを物理的に削除
        let stmt2 = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM messages WHERE sender_id IN (1, 2) AND receiver_id IN (1, 2)",
            vec![],
        );
        db.execute(stmt2).await?;

        Ok(())
    }

    /// テスト用のダミーユーザーをセットアップします
    async fn setup_dummy_users(db: &DatabaseConnection) -> Result<(), DbErr> {
        // ユーザーID 1 の挿入
        let stmt1 = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO users (id, name, email, created_at, updated_at) VALUES (1, 'Test User 1', 'user1@example.com', NOW(), NOW()) ON CONFLICT (id) DO NOTHING",
            vec![],
        );
        db.execute(stmt1).await?;

        // ユーザーID 2 の挿入
        let stmt2 = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO users (id, name, email, created_at, updated_at) VALUES (2, 'Test User 2', 'user2@example.com', NOW(), NOW()) ON CONFLICT (id) DO NOTHING",
            vec![],
        );
        db.execute(stmt2).await?;

        Ok(())
    }

    /// Sets up test users with unique IDs to avoid conflicts
    async fn setup_test_users(db: &DatabaseConnection) -> Result<(i32, i32), DbErr> {
        let test_id = generate_test_id();
        let user1_name = format!("Test User 1 {}", test_id);
        let user2_name = format!("Test User 2 {}", test_id);

        // Create user 1
        let stmt1 = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO users (name, email, created_at, updated_at) 
             VALUES ($1, $2, NOW(), NOW()) RETURNING id",
            vec![user1_name.into(), format!("user1_{}@example.com", test_id).into()],
        );
        let result1 = db.query_one(stmt1).await?;
        let user1_id = result1.unwrap().try_get::<i32>("", "id").unwrap();

        // Create user 2
        let stmt2 = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO users (name, email, created_at, updated_at) 
             VALUES ($1, $2, NOW(), NOW()) RETURNING id",
            vec![user2_name.into(), format!("user2_{}@example.com", test_id).into()],
        );
        let result2 = db.query_one(stmt2).await?;
        let user2_id = result2.unwrap().try_get::<i32>("", "id").unwrap();

        Ok((user1_id, user2_id))
    }

    /// テスト用のダミーメッセージをセットアップします
    async fn setup_dummy_message(db: &DatabaseConnection) -> Result<i32, DbErr> {
        // メッセージの挿入 - NOW()をそのまま使用（タイムゾーン付きのまま）
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO messages (sender_id, receiver_id, content, is_read, created_at, updated_at) VALUES (1, 2, 'Test message for reactions', false, NOW(), NOW()) RETURNING id",
            vec![],
        );
        let result = db.query_one(stmt).await?;
        let message_id = result.unwrap().try_get::<i32>("", "id").unwrap();
        Ok(message_id)
    }

    /// Creates a test message with guaranteed unique IDs
    async fn setup_test_message(db: &DatabaseConnection, sender_id: i32, receiver_id: i32) -> Result<i32, DbErr> {
        let test_id = generate_test_id();
        let content = format!("Test message for reactions {}", test_id);

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO messages (sender_id, receiver_id, content, is_read, created_at, updated_at) 
             VALUES ($1, $2, $3, false, NOW(), NOW()) RETURNING id",
            vec![sender_id.into(), receiver_id.into(), content.into()],
        );

        let result = db.query_one(stmt).await?;
        let message_id = result.unwrap().try_get::<i32>("", "id").unwrap();
        Ok(message_id)
    }

    /// Cleans up test data for specific IDs
    async fn cleanup_specific_test_data(
        db: &DatabaseConnection,
        message_id: i32,
        user1_id: i32,
        user2_id: i32
    ) -> Result<(), DbErr> {
        // Delete reactions for this specific message
        let stmt1 = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM reactions WHERE message_id = $1",
            vec![message_id.into()],
        );
        db.execute(stmt1).await?;

        // Delete this specific message
        let stmt2 = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM messages WHERE id = $1",
            vec![message_id.into()],
        );
        db.execute(stmt2).await?;

        // Delete specific test users
        let stmt3 = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "DELETE FROM users WHERE id IN ($1, $2)",
            vec![user1_id.into(), user2_id.into()],
        );
        db.execute(stmt3).await?;

        Ok(())
    }

    /// テスト用のリアクションタイプをセットアップします
    async fn setup_reaction_types(db: &DatabaseConnection) -> Result<(), DbErr> {
        // リアクションタイプを挿入 - NOW()をそのまま使用（タイムゾーン付きのまま）
        let types = [
            (1, "いいね", "👍"),
            (2, "わかる", "🤝"),
            (3, "応援してる", "🎉"),
            (4, "おつかれさま", "🙏"),
            (5, "たしかに", "💡"),
            (6, "すごい", "✨"),
            (7, "笑った", "😂"),
        ];

        for (id, name, emoji) in types {
            let stmt = Statement::from_sql_and_values(
                DbBackend::Postgres,
                "INSERT INTO reaction_types (id, name, emoji, created_at, updated_at) VALUES ($1, $2, $3, NOW(), NOW()) ON CONFLICT (id) DO UPDATE SET name = $2, emoji = $3",
                vec![id.into(), name.into(), emoji.into()],
            );
            db.execute(stmt).await?;
        }

        Ok(())
    }

    /// データの修正用
    async fn fix_timestamp_types(db: &DatabaseConnection) -> Result<(), DbErr> {
        // 型変換用クエリ（問題が解決しない場合の追加対策）
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "ALTER TABLE reaction_types ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE USING created_at::TIMESTAMP",
            vec![],
        );
        let _ = db.execute(stmt).await;

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "ALTER TABLE reaction_types ALTER COLUMN updated_at TYPE TIMESTAMP WITHOUT TIME ZONE USING updated_at::TIMESTAMP",
            vec![],
        );
        let _ = db.execute(stmt).await;

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "ALTER TABLE reactions ALTER COLUMN created_at TYPE TIMESTAMP WITHOUT TIME ZONE USING created_at::TIMESTAMP",
            vec![],
        );
        let _ = db.execute(stmt).await;

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "ALTER TABLE reactions ALTER COLUMN updated_at TYPE TIMESTAMP WITHOUT TIME ZONE USING updated_at::TIMESTAMP",
            vec![],
        );
        let _ = db.execute(stmt).await;

        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "ALTER TABLE reactions ALTER COLUMN deleted_at TYPE TIMESTAMP WITHOUT TIME ZONE USING deleted_at::TIMESTAMP",
            vec![],
        );
        let _ = db.execute(stmt).await;

        Ok(())
    }

    #[tokio::test]
    async fn test_count_reactions_by_type() {
        let db = setup_db().await;

        // Fix timestamp types if needed
        let _ = fix_timestamp_types(&db).await;

        // Setup reaction types (this is shared across tests and should be fine)
        setup_reaction_types(&db).await.expect("Failed to setup reaction types");

        // Create unique test users for this test
        let (user1_id, user2_id) = setup_test_users(&db)
            .await
            .expect("Failed to create test users");

        println!("Created test users with IDs: {} and {}", user1_id, user2_id);

        // Create a unique test message
        let message_id = setup_test_message(&db, user1_id, user2_id)
            .await
            .expect("Failed to create test message");

        println!("Created test message with ID: {}", message_id);

        let repo = PgReactionRepository::new(db.clone());

        // Add reactions to the message
        repo.add_reaction(user1_id, message_id, 1)
            .await
            .expect("Failed to add reaction 1");

        repo.add_reaction(user2_id, message_id, 2)
            .await
            .expect("Failed to add reaction 2");

        repo.add_reaction(user1_id, message_id, 3)
            .await
            .expect("Failed to add reaction 3");

        // Get reaction counts
        let counts = repo
            .count_reactions_by_type(message_id)
            .await
            .expect("Failed to count reactions by type");

        println!("Count result length: {}", counts.len());
        for (reaction_type, count) in &counts {
            println!("Type: {}, Count: {}", reaction_type.id, count);
        }

        assert_eq!(counts.len(), 3, "Should have 3 different reaction types");

        // Verify each reaction type count
        let mut found_types = [false; 3];
        for (reaction_type, count) in &counts {
            match reaction_type.id {
                1 => {
                    assert_eq!(*count, 1, "Like count should be 1");
                    found_types[0] = true;
                }
                2 => {
                    assert_eq!(*count, 1, "Understand count should be 1");
                    found_types[1] = true;
                }
                3 => {
                    assert_eq!(*count, 1, "Support count should be 1");
                    found_types[2] = true;
                }
                _ => println!("Found unexpected reaction type ID: {}", reaction_type.id),
            }
        }

        // Ensure all expected types were found
        assert!(
            found_types.iter().all(|&found| found),
            "Not all reaction types were found"
        );

        // Clean up specific test data
        cleanup_specific_test_data(&db, message_id, user1_id, user2_id)
            .await
            .expect("Failed to cleanup test data");
    }

    #[tokio::test]
    async fn test_add_and_get_reaction() {
        let db = setup_db().await;
        // Fix timestamp types if needed
        let _ = fix_timestamp_types(&db).await;

        // Setup reaction types (shared across tests)
        setup_reaction_types(&db).await.expect("Failed to setup reaction types");

        // Create unique test users for this test
        let (user1_id, user2_id) = setup_test_users(&db)
            .await
            .expect("Failed to create test users");

        println!("Created test users with IDs: {} and {}", user1_id, user2_id);

        // Create a unique test message
        let message_id = setup_test_message(&db, user1_id, user2_id)
            .await
            .expect("Failed to create test message");

        println!("Created test message with ID: {}", message_id);

        let repo = PgReactionRepository::new(db.clone());

        // Add a reaction
        let reaction = repo
            .add_reaction(user1_id, message_id, 1) // いいね
            .await
            .expect("Failed to add reaction");

        assert_eq!(reaction.user_id, user1_id);
        assert_eq!(reaction.message_id, message_id);
        assert_eq!(reaction.reaction_type_id, 1);

        // Get reactions for the message
        let reactions = repo
            .get_reactions_for_message(message_id)
            .await
            .expect("Failed to get reactions for message");

        println!("Found {} reactions for message ID {}", reactions.len(), message_id);

        assert_eq!(reactions.len(), 1, "Should find exactly one reaction");
        assert_eq!(reactions[0].user_id, user1_id);
        assert_eq!(reactions[0].reaction_type_id, 1);

        // Clean up specific test data
        cleanup_specific_test_data(&db, message_id, user1_id, user2_id)
            .await
            .expect("Failed to cleanup test data");
    }

    #[tokio::test]
    async fn test_remove_reaction() {
        let db = setup_db().await;
        // Fix timestamp types if needed
        let _ = fix_timestamp_types(&db).await;

        // Setup reaction types (shared across tests)
        setup_reaction_types(&db).await.expect("Failed to setup reaction types");

        // Create unique test users for this test
        let (user1_id, user2_id) = setup_test_users(&db)
            .await
            .expect("Failed to create test users");

        println!("Created test users with IDs: {} and {}", user1_id, user2_id);

        // Create a unique test message
        let message_id = setup_test_message(&db, user1_id, user2_id)
            .await
            .expect("Failed to create test message");

        println!("Created test message with ID: {}", message_id);

        let repo = PgReactionRepository::new(db.clone());

        // Add a reaction
        repo.add_reaction(user1_id, message_id, 1)
            .await
            .expect("Failed to add reaction");

        // Remove the reaction
        let removed_count = repo
            .remove_reaction(user1_id, message_id, Some(1))
            .await
            .expect("Failed to remove reaction");

        assert_eq!(removed_count, 1, "Should have removed exactly one reaction");

        // Check that no reactions remain
        let reactions = repo
            .get_reactions_for_message(message_id)
            .await
            .expect("Failed to get reactions for message");

        assert_eq!(reactions.len(), 0, "All reactions should be removed");

        // Clean up specific test data
        cleanup_specific_test_data(&db, message_id, user1_id, user2_id)
            .await
            .expect("Failed to cleanup test data");
    }

    #[tokio::test]
    async fn test_list_reaction_types() {
        let db = setup_db().await;
        // 型の不一致を解決するためのスキーマ修正（初回のみ実行）
        let _ = fix_timestamp_types(&db).await;

        // リアクションタイプをセットアップ
        setup_reaction_types(&db)
            .await
            .expect("Failed to setup reaction types");

        let repo = PgReactionRepository::new(db.clone());

        // リアクションタイプの一覧を取得
        let types = repo
            .list_reaction_types()
            .await
            .expect("list_reaction_types に失敗しました");

        // 7種類のリアクションタイプがあることを確認
        assert!(types.len() >= 7, "リアクションタイプの数が7未満です");

        // 各リアクションタイプの名前を確認
        let type_names: Vec<String> = types.iter().map(|t| t.name.clone()).collect();
        assert!(
            type_names.contains(&"いいね".to_string()),
            "「いいね」リアクションタイプがありません"
        );
        assert!(
            type_names.contains(&"わかる".to_string()),
            "「わかる」リアクションタイプがありません"
        );
        assert!(
            type_names.contains(&"応援してる".to_string()),
            "「応援してる」リアクションタイプがありません"
        );
        assert!(
            type_names.contains(&"おつかれさま".to_string()),
            "「おつかれさま」リアクションタイプがありません"
        );
        assert!(
            type_names.contains(&"たしかに".to_string()),
            "「たしかに」リアクションタイプがありません"
        );
        assert!(
            type_names.contains(&"すごい".to_string()),
            "「すごい」リアクションタイプがありません"
        );
        assert!(
            type_names.contains(&"笑った".to_string()),
            "「笑った」リアクションタイプがありません"
        );
    }
}