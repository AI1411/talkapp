use sea_orm::DatabaseConnection;
use crate::domain::entity::post::Model as Post;
use async_trait::async_trait;
use sea_orm::entity::prelude::*;

#[async_trait]
pub trait PostRepositoryTrait {
    async fn find_all(&self) -> Result<Vec<Post>, DbErr>;
    async fn find_by_id(&self, id: i32) -> Result<Option<Post>, DbErr>;
    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Post>, DbErr>;
    async fn insert(&self, post: &Post) -> Result<Post, DbErr>;
    async fn update(&self, post: &Post) -> Result<Post, DbErr>;
    async fn delete(&self, post: &Post) -> Result<Post, DbErr>;
}

pub struct PgPostRepository {
    db: DatabaseConnection,
}

