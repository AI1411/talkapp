use crate::domain::entity::users::Model as User;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository {
    async fn get_by_id(&self, id: i32) -> Result<User, sqlx::Error>;
    async fn list(&self) -> Result<Vec<User>, sqlx::Error>;
    async fn create(
        &self,
        name: String,
        email: String,
        description: Option<String>,
        age: Option<i32>,
        gender: Option<String>,
        address: Option<String>,
    ) -> Result<User, sqlx::Error>;
    async fn update(
        &self,
        id: i32,
        name: Option<String>,
        email: Option<String>,
        description: Option<String>,
        age: Option<i32>,
        gender: Option<String>,
        address: Option<String>,
    ) -> Result<User, sqlx::Error>;
    async fn delete(&self, id: i32) -> Result<User, sqlx::Error>;
    async fn hard_delete(&self, id: i32) -> Result<(), sqlx::Error>;
}
