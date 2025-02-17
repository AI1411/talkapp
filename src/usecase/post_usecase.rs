use crate::domain::entity::post::Model as Post;
use crate::domain::repository::post::PostRepository;
use async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
pub trait PostUseCase {
    async fn create_post(&self, body: String, user_id: i32) -> Result<Post, DbErr>;
    async fn get_post(&self, id: i32) -> Result<Option<Post>, DbErr>;
    async fn list_posts(&self) -> Result<Vec<Post>, DbErr>;
    async fn delete_post(&self, id: i32) -> Result<(), DbErr>;
}

pub struct PostUseCaseImpl<R> {
    repository: R,
}

impl<R: PostRepository> PostUseCaseImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: PostRepository + Send + Sync> PostUseCase for PostUseCaseImpl<R> {
    async fn create_post(&self, body: String, user_id: i32) -> Result<Post, DbErr> {
        self.repository.insert(body, user_id).await
    }

    async fn get_post(&self, id: i32) -> Result<Option<Post>, DbErr> {
        self.repository.get_by_id(id).await
    }

    async fn list_posts(&self) -> Result<Vec<Post>, DbErr> {
        self.repository.find_all().await
    }

    async fn delete_post(&self, id: i32) -> Result<(), DbErr> {
        self.repository.delete(id).await
    }
}
