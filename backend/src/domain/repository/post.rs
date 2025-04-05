use crate::domain::entity::post::Model as Post;
use async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
pub trait PostRepository {
    async fn find_all(&self) -> Result<Vec<Post>, DbErr>;
    async fn get_by_id(&self, id: i32) -> Result<Option<Post>, DbErr>;
    async fn insert(&self, body: String, user_id: i32) -> Result<Post, DbErr>;
    async fn delete(&self, id: i32) -> Result<(), DbErr>;
}
