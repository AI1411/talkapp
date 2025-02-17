// repository.rs
use crate::domain::entity::users::Model as User;
use crate::domain::entity::users::{self, Entity as Users};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, EntityTrait, Set};
use crate::domain::repository::user::UserRepository;

/// PgUserRepository の実装
pub struct PgUserRepository {
    pool: DatabaseConnection,
}

impl PgUserRepository {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn get_by_id(&self, id: i32) -> Result<User, sqlx::Error> {
        Users::find_by_id(id)
            .one(&self.pool)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?
            .ok_or(sqlx::Error::RowNotFound)
    }

    async fn list(&self) -> Result<Vec<User>, sqlx::Error> {
        Users::find()
            .all(&self.pool)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))
    }

    async fn create(
        &self,
        name: String,
        email: String,
        description: Option<String>,
        age: Option<i32>,
        gender: Option<String>,
        address: Option<String>,
    ) -> Result<User, sqlx::Error> {
        let now = chrono::Utc::now().naive_utc();
        let user = users::ActiveModel {
            id: NotSet,
            name: Set(name),
            email: Set(email),
            description: Set(description),
            age: Set(age),
            gender: Set(gender),
            address: Set(address),
            created_at: Default::default(),
            updated_at: Default::default(),
            deleted_at: Default::default(),
        };

        user.insert(&self.pool)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))
    }

    async fn update(
        &self,
        id: i32,
        name: Option<String>,
        email: Option<String>,
        description: Option<String>,
        age: Option<i32>,
        gender: Option<String>,
        address: Option<String>,
    ) -> Result<User, sqlx::Error> {
        let user = Users::find_by_id(id)
            .one(&self.pool)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?
            .ok_or(sqlx::Error::RowNotFound)?;

        let mut user: users::ActiveModel = user.into();

        if let Some(name) = name {
            user.name = Set(name);
        }
        if let Some(email) = email {
            user.email = Set(email);
        }
        if let Some(desc) = description {
            user.description = Set(Some(desc));
        }
        if let Some(age) = age {
            user.age = Set(Some(age));
        }
        if let Some(gender) = gender {
            user.gender = Set(Some(gender));
        }
        if let Some(addr) = address {
            user.address = Set(Some(addr));
        }
        user.updated_at = Set(chrono::Utc::now().naive_utc());

        user.update(&self.pool)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))
    }

    async fn delete(&self, id: i32) -> Result<User, sqlx::Error> {
        let user = self.get_by_id(id).await?;
        let mut user: users::ActiveModel = user.into();

        user.deleted_at = Set(Some(chrono::Utc::now().naive_utc()));
        user.update(&self.pool)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))
    }

    async fn hard_delete(&self, id: i32) -> Result<(), sqlx::Error> {
        Users::delete_by_id(id)
            .exec(&self.pool)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))?;
        Ok(())
    }
}

/// モック実装（テスト用）
#[cfg(test)]
pub mod mock {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    pub struct MockUserRepository {
        pub users: Mutex<HashMap<i32, User>>,
        pub next_id: Mutex<i32>,
    }

    impl MockUserRepository {
        pub fn new() -> Self {
            Self {
                users: Mutex::new(HashMap::new()),
                next_id: Mutex::new(1),
            }
        }
    }

    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn get_by_id(&self, id: i32) -> Result<User, sqlx::Error> {
            let users = self.users.lock().unwrap();
            users
                .get(&id)
                .cloned()
                .ok_or_else(|| sqlx::Error::RowNotFound)
        }

        async fn list(&self) -> Result<Vec<User>, sqlx::Error> {
            let users = self.users.lock().unwrap();
            Ok(users.values().cloned().collect())
        }

        async fn create(
            &self,
            name: String,
            email: String,
            description: Option<String>,
            age: Option<i32>,
            gender: Option<String>,
            address: Option<String>,
        ) -> Result<User, sqlx::Error> {
            let mut next_id = self.next_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;

            let now = chrono::Utc::now().naive_utc();
            let user = User {
                id,
                name,
                email,
                description,
                age,
                gender,
                address,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };

            let mut users = self.users.lock().unwrap();
            users.insert(id, user.clone());
            Ok(user)
        }

        async fn update(
            &self,
            id: i32,
            name: Option<String>,
            email: Option<String>,
            description: Option<String>,
            age: Option<i32>,
            gender: Option<String>,
            address: Option<String>,
        ) -> Result<User, sqlx::Error> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(&id) {
                if let Some(name) = name {
                    user.name = name;
                }
                if let Some(email) = email {
                    user.email = email;
                }
                if let Some(desc) = description {
                    user.description = Some(desc);
                }
                if let Some(age) = age {
                    user.age = Some(age);
                }
                if let Some(gender) = gender {
                    user.gender = Some(gender);
                }
                if let Some(addr) = address {
                    user.address = Some(addr);
                }
                user.updated_at = chrono::Utc::now().naive_utc();
                Ok(user.clone())
            } else {
                Err(sqlx::Error::RowNotFound)
            }
        }

        async fn delete(&self, id: i32) -> Result<User, sqlx::Error> {
            let mut users = self.users.lock().unwrap();
            if let Some(user) = users.get_mut(&id) {
                user.deleted_at = Some(chrono::Utc::now().naive_utc());
                Ok(user.clone())
            } else {
                Err(sqlx::Error::RowNotFound)
            }
        }

        async fn hard_delete(&self, id: i32) -> Result<(), sqlx::Error> {
            let mut users = self.users.lock().unwrap();
            if users.remove(&id).is_some() {
                Ok(())
            } else {
                Err(sqlx::Error::RowNotFound)
            }
        }
    }
}

/// テストコード
#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use sea_orm::Database;
    use std::env;

    async fn setup_test_db() -> DatabaseConnection {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Database::connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    mod pg_repository_tests {
        use super::*;

        async fn create_test_user(repo: &PgUserRepository) -> User {
            repo.create(
                "Test User".to_string(),
                "test@example.com".to_string(),
                Some("Test Description".to_string()),
                Some(25),
                Some("Male".to_string()),
                None, // address
            )
            .await
            .expect("Failed to create test user")
        }

        #[tokio::test]
        async fn test_create_user() {
            let pool = setup_test_db().await;
            let repo = PgUserRepository::new(pool);

            let user = repo
                .create(
                    "Test User".to_string(),
                    "test@example.com".to_string(),
                    Some("Test Description".to_string()),
                    Some(25),
                    Some("Male".to_string()),
                    None,
                )
                .await
                .expect("Failed to create user");

            assert_eq!(user.name, "Test User");
            assert_eq!(user.description, Some("Test Description".to_string()));
            assert_eq!(user.age, Some(25));
            assert_eq!(user.gender, Some("Male".to_string()));
            assert!(user.deleted_at.is_none());

            repo.hard_delete(user.id).await.expect("Failed to cleanup");
        }

        #[tokio::test]
        async fn test_get_user_by_id() {
            let pool = setup_test_db().await;
            let repo = PgUserRepository::new(pool);

            let created_user = create_test_user(&repo).await;

            let retrieved_user = repo
                .get_by_id(created_user.id)
                .await
                .expect("Failed to get user");

            assert_eq!(retrieved_user.id, created_user.id);
            assert_eq!(retrieved_user.name, created_user.name);
            assert_eq!(retrieved_user.description, created_user.description);
            assert_eq!(retrieved_user.age, created_user.age);
            assert_eq!(retrieved_user.gender, created_user.gender);

            repo.hard_delete(created_user.id)
                .await
                .expect("Failed to cleanup");
        }

        #[tokio::test]
        async fn test_list_users() {
            let pool = setup_test_db().await;
            let repo = PgUserRepository::new(pool);

            let user1 = create_test_user(&repo).await;
            let user2 = repo
                .create(
                    "Another User".to_string(),
                    "another@example.com".to_string(),
                    Some("Another Description".to_string()),
                    Some(30),
                    Some("Female".to_string()),
                    None,
                )
                .await
                .expect("Failed to create second user");

            let users = repo.list().await.expect("Failed to list users");

            assert!(users.len() >= 2);
            assert!(users.iter().any(|u| u.id == user1.id));
            assert!(users.iter().any(|u| u.id == user2.id));

            repo.hard_delete(user1.id).await.expect("Failed to cleanup");
            repo.hard_delete(user2.id).await.expect("Failed to cleanup");
        }

        #[tokio::test]
        async fn test_update_user() {
            let pool = setup_test_db().await;
            let repo = PgUserRepository::new(pool);

            let created_user = create_test_user(&repo).await;

            let updated_user = repo
                .update(
                    created_user.id,
                    Some("Updated Name".to_string()),
                    Some("updated@example.com".to_string()),
                    Some("Updated Description".to_string()),
                    Some(26),
                    Some("Female".to_string()),
                    None, // address
                )
                .await
                .expect("Failed to update user");

            assert_eq!(updated_user.id, created_user.id);
            assert_eq!(updated_user.name, "Updated Name");
            assert_eq!(
                updated_user.description,
                Some("Updated Description".to_string())
            );
            assert_eq!(updated_user.age, Some(26));
            assert_eq!(updated_user.gender, Some("Female".to_string()));
            assert!(updated_user.updated_at > created_user.updated_at);

            repo.hard_delete(created_user.id)
                .await
                .expect("Failed to cleanup");
        }

        #[tokio::test]
        async fn test_partial_update() {
            let pool = setup_test_db().await;
            let repo = PgUserRepository::new(pool);

            let created_user = create_test_user(&repo).await;

            let updated_user = repo
                .update(
                    created_user.id,
                    Some("Updated Name".to_string()),
                    None, // email は更新しない
                    None, // description は更新しない
                    None, // age は更新しない
                    None, // gender は更新しない
                    None, // address は更新しない
                )
                .await
                .expect("Failed to partially update user");

            assert_eq!(updated_user.name, "Updated Name");
            assert_eq!(updated_user.description, created_user.description);
            assert_eq!(updated_user.age, created_user.age);
            assert_eq!(updated_user.gender, created_user.gender);

            repo.hard_delete(created_user.id)
                .await
                .expect("Failed to cleanup");
        }

        #[tokio::test]
        async fn test_delete_user() {
            let pool = setup_test_db().await;
            let repo = PgUserRepository::new(pool);

            let created_user = create_test_user(&repo).await;

            let deleted_user = repo
                .delete(created_user.id)
                .await
                .expect("Failed to delete user");

            assert_eq!(deleted_user.id, created_user.id);
            assert!(deleted_user.deleted_at.is_some());

            repo.hard_delete(created_user.id)
                .await
                .expect("Failed to hard delete user");

            let result = repo.get_by_id(created_user.id).await;
            assert!(result.is_err());
        }
    }

    mod mock_repository_tests {
        use super::*;

        #[tokio::test]
        async fn test_mock_create_and_get() {
            let repo = mock::MockUserRepository::new();

            let user = repo
                .create(
                    "Test User".to_string(),
                    "test@example.com".to_string(),
                    Some("Test Description".to_string()),
                    Some(25),
                    Some("Male".to_string()),
                    None,
                )
                .await
                .expect("Failed to create user");

            assert_eq!(user.name, "Test User");

            let retrieved = repo.get_by_id(user.id).await.expect("Failed to get user");
            assert_eq!(retrieved.id, user.id);
            assert_eq!(retrieved.name, "Test User");
        }

        #[tokio::test]
        async fn test_mock_list_users() {
            let repo = mock::MockUserRepository::new();

            let user1 = repo
                .create(
                    "User 1".to_string(),
                    "user1@example.com".to_string(),
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .expect("Failed to create user 1");
            let user2 = repo
                .create(
                    "User 2".to_string(),
                    "user2@example.com".to_string(),
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .expect("Failed to create user 2");

            let users = repo.list().await.expect("Failed to list users");
            assert_eq!(users.len(), 2);
            assert!(users.iter().any(|u| u.id == user1.id));
            assert!(users.iter().any(|u| u.id == user2.id));
        }

        #[tokio::test]
        async fn test_mock_update() {
            let repo = mock::MockUserRepository::new();

            let user = repo
                .create(
                    "Initial Name".to_string(),
                    "initial@example.com".to_string(),
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .expect("Failed to create user");

            let updated = repo
                .update(
                    user.id,
                    Some("Updated Name".to_string()),
                    None, // email は更新しない
                    None, // description は更新しない
                    None, // age は更新しない
                    None, // gender は更新しない
                    None, // address は更新しない
                )
                .await
                .expect("Failed to update user");

            assert_eq!(updated.name, "Updated Name");
        }

        #[tokio::test]
        async fn test_mock_delete() {
            let repo = mock::MockUserRepository::new();

            let user = repo
                .create(
                    "Test User".to_string(),
                    "test@example.com".to_string(),
                    None,
                    None,
                    None,
                    None,
                )
                .await
                .expect("Failed to create user");

            let deleted = repo.delete(user.id).await.expect("Failed to delete user");
            assert!(deleted.deleted_at.is_some());

            repo.hard_delete(user.id)
                .await
                .expect("Failed to hard delete");
            let result = repo.get_by_id(user.id).await;
            assert!(matches!(result, Err(sqlx::Error::RowNotFound)));
        }

        #[tokio::test]
        async fn test_mock_not_found() {
            let repo = mock::MockUserRepository::new();
            let result = repo.get_by_id(999).await;
            assert!(matches!(result, Err(sqlx::Error::RowNotFound)));
        }
    }
}
