use crate::domain::entity::{reactions, reaction_types};
use crate::domain::repository::reaction::ReactionRepository;
use async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
pub trait ReactionUseCase {
    /// メッセージにリアクションを追加します。
    async fn add_reaction(
        &self,
        user_id: i32,
        message_id: i32,
        reaction_type_id: i32,
    ) -> Result<reactions::Model, DbErr>;

    /// メッセージからリアクションを削除します。
    async fn remove_reaction(
        &self,
        user_id: i32,
        message_id: i32,
        reaction_type_id: Option<i32>,
    ) -> Result<i32, DbErr>;

    /// メッセージに付けられたリアクションを取得します。
    async fn get_reactions_for_message(
        &self,
        message_id: i32,
    ) -> Result<Vec<reactions::Model>, DbErr>;

    /// メッセージに付けられたリアクションを集計します。
    async fn count_reactions_by_type(
        &self,
        message_id: i32,
    ) -> Result<Vec<(reaction_types::Model, i64)>, DbErr>;

    /// 利用可能なリアクションの種類を全て取得します。
    async fn list_reaction_types(&self) -> Result<Vec<reaction_types::Model>, DbErr>;

    /// 指定されたIDのリアクションタイプを取得します。
    async fn get_reaction_type(&self, id: i32) -> Result<Option<reaction_types::Model>, DbErr>;
}

pub struct ReactionUseCaseImpl<R> {
    repository: R,
}

impl<R: ReactionRepository> ReactionUseCaseImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: ReactionRepository + Send + Sync> ReactionUseCase for ReactionUseCaseImpl<R> {
    async fn add_reaction(
        &self,
        user_id: i32,
        message_id: i32,
        reaction_type_id: i32,
    ) -> Result<reactions::Model, DbErr> {
        self.repository.add_reaction(user_id, message_id, reaction_type_id).await
    }

    async fn remove_reaction(
        &self,
        user_id: i32,
        message_id: i32,
        reaction_type_id: Option<i32>,
    ) -> Result<i32, DbErr> {
        self.repository.remove_reaction(user_id, message_id, reaction_type_id).await
    }

    async fn get_reactions_for_message(
        &self,
        message_id: i32,
    ) -> Result<Vec<reactions::Model>, DbErr> {
        self.repository.get_reactions_for_message(message_id).await
    }

    async fn count_reactions_by_type(
        &self,
        message_id: i32,
    ) -> Result<Vec<(reaction_types::Model, i64)>, DbErr> {
        self.repository.count_reactions_by_type(message_id).await
    }

    async fn list_reaction_types(&self) -> Result<Vec<reaction_types::Model>, DbErr> {
        self.repository.list_reaction_types().await
    }

    async fn get_reaction_type(&self, id: i32) -> Result<Option<reaction_types::Model>, DbErr> {
        self.repository.get_reaction_type(id).await
    }
}
