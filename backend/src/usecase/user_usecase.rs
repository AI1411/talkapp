use crate::domain::entity::users::Model as User;
use crate::domain::repository::user::UserRepository;
use async_trait::async_trait;

#[async_trait]
pub trait UserUseCase {
    async fn create_user(
        &self,
        name: String,
        email: String,
        description: Option<String>,
        age: Option<i32>,
        gender: Option<String>,
        address: Option<String>,
    ) -> Result<User, sqlx::Error>;

    async fn get_user(&self, id: i32) -> Result<User, sqlx::Error>;

    async fn list_users(&self) -> Result<Vec<User>, sqlx::Error>;

    // 修正: sex ではなく gender とし、email, address も含む全パラメータを渡す
    async fn update_user(
        &self,
        id: i32,
        name: Option<String>,
        email: Option<String>,
        description: Option<String>,
        age: Option<i32>,
        gender: Option<String>,
        address: Option<String>,
    ) -> Result<User, sqlx::Error>;

    async fn delete_user(&self, id: i32) -> Result<User, sqlx::Error>;
}

pub struct UserUseCaseImpl<R> {
    repository: R,
}

impl<R: UserRepository> UserUseCaseImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: UserRepository + Send + Sync> UserUseCase for UserUseCaseImpl<R> {
    async fn create_user(
        &self,
        name: String,
        email: String,
        description: Option<String>,
        age: Option<i32>,
        gender: Option<String>,
        address: Option<String>,
    ) -> Result<User, sqlx::Error> {
        self.repository
            .create(name, email, description, age, gender, address)
            .await
    }

    async fn get_user(&self, id: i32) -> Result<User, sqlx::Error> {
        self.repository.get_by_id(id).await
    }

    async fn list_users(&self) -> Result<Vec<User>, sqlx::Error> {
        self.repository.list().await
    }

    async fn update_user(
        &self,
        id: i32,
        name: Option<String>,
        email: Option<String>,
        description: Option<String>,
        age: Option<i32>,
        gender: Option<String>,
        address: Option<String>,
    ) -> Result<User, sqlx::Error> {
        self.repository
            .update(id, name, email, description, age, gender, address)
            .await
    }

    async fn delete_user(&self, id: i32) -> Result<User, sqlx::Error> {
        self.repository.delete(id).await
    }
}
