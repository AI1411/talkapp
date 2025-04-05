use crate::domain::entity::{group_members, group_messages, groups};
use crate::domain::repository::group::GroupRepository;
use sea_orm::DbErr;

#[async_trait::async_trait]
pub trait GroupUseCase {
    async fn create_group(
        &self,
        name: String,
        description: Option<String>,
        creator_id: i32,
        initial_member_ids: Vec<i32>,
    ) -> Result<groups::Model, DbErr>;

    async fn get_group(&self, group_id: i32) -> Result<Option<groups::Model>, DbErr>;

    async fn list_groups(
        &self,
        user_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<groups::Model>, i32), DbErr>;

    async fn update_group(
        &self,
        group_id: i32,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<groups::Model, DbErr>;

    async fn delete_group(&self, group_id: i32) -> Result<bool, DbErr>;

    async fn add_group_member(
        &self,
        group_id: i32,
        user_id: i32,
        role: String,
    ) -> Result<group_members::Model, DbErr>;

    async fn remove_group_member(&self, group_id: i32, user_id: i32) -> Result<bool, DbErr>;

    async fn list_group_members(
        &self,
        group_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<(group_members::Model, Option<crate::domain::entity::users::Model>)>, i32), DbErr>;

    async fn send_group_message(
        &self,
        group_id: i32,
        sender_id: i32,
        content: String,
    ) -> Result<group_messages::Model, DbErr>;

    async fn list_group_messages(
        &self,
        group_id: i32,
        user_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<(group_messages::Model, Vec<i32>)>, i32, i32), DbErr>;

    async fn mark_group_message_as_read(
        &self,
        group_id: i32,
        message_id: i32,
        user_id: i32,
    ) -> Result<bool, DbErr>;

    async fn delete_group_message(&self, message_id: i32) -> Result<bool, DbErr>;
}

pub struct GroupUseCaseImpl<R> {
    repository: R,
}

impl<R: GroupRepository> GroupUseCaseImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait::async_trait]
impl<R: GroupRepository + Send + Sync> GroupUseCase for GroupUseCaseImpl<R> {
    async fn create_group(
        &self,
        name: String,
        description: Option<String>,
        creator_id: i32,
        initial_member_ids: Vec<i32>,
    ) -> Result<groups::Model, DbErr> {
        self.repository
            .create_group(name, description, creator_id, initial_member_ids)
            .await
    }

    async fn get_group(&self, group_id: i32) -> Result<Option<groups::Model>, DbErr> {
        self.repository.get_group(group_id).await
    }

    async fn list_groups(
        &self,
        user_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<groups::Model>, i32), DbErr> {
        self.repository.list_groups(user_id, page, per_page).await
    }

    async fn update_group(
        &self,
        group_id: i32,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<groups::Model, DbErr> {
        self.repository
            .update_group(group_id, name, description)
            .await
    }

    async fn delete_group(&self, group_id: i32) -> Result<bool, DbErr> {
        self.repository.delete_group(group_id).await
    }

    async fn add_group_member(
        &self,
        group_id: i32,
        user_id: i32,
        role: String,
    ) -> Result<group_members::Model, DbErr> {
        self.repository
            .add_group_member(group_id, user_id, role)
            .await
    }

    async fn remove_group_member(&self, group_id: i32, user_id: i32) -> Result<bool, DbErr> {
        self.repository.remove_group_member(group_id, user_id).await
    }

    async fn list_group_members(
        &self,
        group_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<(group_members::Model, Option<crate::domain::entity::users::Model>)>, i32), DbErr> {
        self.repository
            .list_group_members(group_id, page, per_page)
            .await
    }

    async fn send_group_message(
        &self,
        group_id: i32,
        sender_id: i32,
        content: String,
    ) -> Result<group_messages::Model, DbErr> {
        self.repository
            .send_group_message(group_id, sender_id, content)
            .await
    }

    async fn list_group_messages(
        &self,
        group_id: i32,
        user_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<(group_messages::Model, Vec<i32>)>, i32, i32), DbErr> {
        self.repository
            .list_group_messages(group_id, user_id, page, per_page)
            .await
    }

    async fn mark_group_message_as_read(
        &self,
        group_id: i32,
        message_id: i32,
        user_id: i32,
    ) -> Result<bool, DbErr> {
        self.repository
            .mark_group_message_as_read(group_id, message_id, user_id)
            .await
    }

    async fn delete_group_message(&self, message_id: i32) -> Result<bool, DbErr> {
        self.repository.delete_group_message(message_id).await
    }
}
