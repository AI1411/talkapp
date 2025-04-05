use crate::domain::entity::messages::Model as Message;
use crate::domain::repository::message::MessageRepository;
use async_trait::async_trait;
use sea_orm::DbErr;

#[async_trait]
pub trait MessageUseCase {
    /// 新規メッセージを送信します。
    async fn send_message(
        &self,
        sender_id: i32,
        receiver_id: i32,
        content: String,
    ) -> Result<Message, DbErr>;

    /// ユーザーのメッセージ一覧を取得します。
    async fn list_messages(
        &self,
        user_id: i32,
        unread_only: bool,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<Message>, i32, i32), DbErr>;

    /// ユーザー間の会話履歴を取得します。
    async fn get_conversation(
        &self,
        user_id: i32,
        peer_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<Message>, i32), DbErr>;

    /// 指定されたメッセージまたはユーザー間のメッセージを既読に更新します。
    async fn mark_as_read(
        &self,
        message_id: Option<i32>,
        message_ids: Vec<i32>,
        from_user_id: Option<i32>,
        to_user_id: Option<i32>,
    ) -> Result<i32, DbErr>;

    /// 指定されたメッセージを削除します。
    async fn delete_message(&self, message_id: i32) -> Result<bool, DbErr>;
}

pub struct MessageUseCaseImpl<R> {
    repository: R,
}

impl<R: MessageRepository> MessageUseCaseImpl<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: MessageRepository + Send + Sync> MessageUseCase for MessageUseCaseImpl<R> {
    async fn send_message(
        &self,
        sender_id: i32,
        receiver_id: i32,
        content: String,
    ) -> Result<Message, DbErr> {
        self.repository
            .send_message(sender_id, receiver_id, content)
            .await
    }

    async fn list_messages(
        &self,
        user_id: i32,
        unread_only: bool,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<Message>, i32, i32), DbErr> {
        self.repository
            .list_messages(user_id, unread_only, page, per_page)
            .await
    }

    async fn get_conversation(
        &self,
        user_id: i32,
        peer_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<Message>, i32), DbErr> {
        self.repository
            .get_conversation(user_id, peer_id, page, per_page)
            .await
    }

    async fn mark_as_read(
        &self,
        message_id: Option<i32>,
        message_ids: Vec<i32>,
        from_user_id: Option<i32>,
        to_user_id: Option<i32>,
    ) -> Result<i32, DbErr> {
        self.repository
            .mark_as_read(message_id, message_ids, from_user_id, to_user_id)
            .await
    }

    async fn delete_message(&self, message_id: i32) -> Result<bool, DbErr> {
        self.repository.delete_message(message_id).await
    }
}
