use crate::message_proto::message_service_server::MessageService;
use crate::message_proto::{
    DeleteMessageRequest, DeleteMessageResponse, GetConversationRequest, GetConversationResponse,
    ListMessagesRequest, ListMessagesResponse, MarkAsReadRequest, MarkAsReadResponse, Message,
    SendMessageRequest, SendMessageResponse,
};
use crate::usecase::message_usecase::MessageUseCase;
use chrono::NaiveDateTime;
use tonic::{Request, Response, Status};
use sea_orm::DbErr;

pub struct MessageHandler<U> {
    usecase: U,
}

impl<U: MessageUseCase> MessageHandler<U> {
    pub fn new(usecase: U) -> Self {
        Self { usecase }
    }

    // NaiveDateTime を文字列に変換するヘルパー関数
    fn format_datetime(dt: NaiveDateTime) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    // メッセージエンティティを Proto メッセージに変換するヘルパー関数
    fn to_proto_message(message: &crate::domain::entity::messages::Model) -> Message {
        Message {
            id: message.id as u64,
            sender_id: message.sender_id as u64,
            receiver_id: message.receiver_id as u64,
            content: message.content.clone(),
            is_read: message.is_read,
            created_at: Self::format_datetime(message.created_at),
            updated_at: Self::format_datetime(message.updated_at),
        }
    }
}

#[tonic::async_trait]
impl<U: MessageUseCase + Send + Sync + 'static> MessageService for MessageHandler<U> {
    async fn send_message(
        &self,
        request: Request<SendMessageRequest>,
    ) -> Result<Response<SendMessageResponse>, Status> {
        let req = request.into_inner();
        let message = self
            .usecase
            .send_message(req.sender_id as i32, req.receiver_id as i32, req.content)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SendMessageResponse {
            message: Some(Self::to_proto_message(&message)),
        }))
    }

    async fn list_messages(
        &self,
        request: Request<ListMessagesRequest>,
    ) -> Result<Response<ListMessagesResponse>, Status> {
        let req = request.into_inner();
        let (messages, total_count, unread_count) = self
            .usecase
            .list_messages(req.user_id as i32, req.unread_only, req.page, req.per_page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_messages = messages.iter().map(|m| Self::to_proto_message(m)).collect();

        Ok(Response::new(ListMessagesResponse {
            messages: proto_messages,
            total_count,
            unread_count,
        }))
    }

    async fn get_conversation(
        &self,
        request: Request<GetConversationRequest>,
    ) -> Result<Response<GetConversationResponse>, Status> {
        let req = request.into_inner();
        let (messages, total_count) = self
            .usecase
            .get_conversation(
                req.user_id as i32,
                req.peer_id as i32,
                req.page,
                req.per_page,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_messages = messages.iter().map(|m| Self::to_proto_message(m)).collect();

        Ok(Response::new(GetConversationResponse {
            messages: proto_messages,
            total_count,
        }))
    }

    async fn mark_as_read(
        &self,
        request: Request<MarkAsReadRequest>,
    ) -> Result<Response<MarkAsReadResponse>, Status> {
        let req = request.into_inner();

        // 単一のメッセージIDまたは複数のメッセージIDを処理
        let message_id = if req.message_id > 0 {
            Some(req.message_id as i32)
        } else {
            None
        };

        let message_ids = req.message_ids.iter().map(|&id| id as i32).collect();

        // from_user_id と to_user_id の処理
        let from_user_id = req.from_user_id.map(|id| id as i32);
        let to_user_id = req.to_user_id.map(|id| id as i32);

        let updated_count = self
            .usecase
            .mark_as_read(message_id, message_ids, from_user_id, to_user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MarkAsReadResponse { updated_count }))
    }

    async fn delete_message(
        &self,
        request: Request<DeleteMessageRequest>,
    ) -> Result<Response<DeleteMessageResponse>, Status> {
        let req = request.into_inner();
        let success = self
            .usecase
            .delete_message(req.message_id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteMessageResponse { success }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::messages;
    use crate::usecase::message_usecase::MessageUseCase;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use mockall::predicate::*;
    use mockall::*;

    // MessageUseCaseのモック
    mock! {
        pub MessageUseCase {}

        #[async_trait]
        impl MessageUseCase for MessageUseCase {
            async fn send_message(
                &self,
                sender_id: i32,
                receiver_id: i32,
                content: String,
            ) -> Result<messages::Model, DbErr>;

            async fn list_messages(
                &self,
                user_id: i32,
                unread_only: bool,
                page: i32,
                per_page: i32,
            ) -> Result<(Vec<messages::Model>, i32, i32), DbErr>;

            async fn get_conversation(
                &self,
                user_id: i32,
                peer_id: i32,
                page: i32,
                per_page: i32,
            ) -> Result<(Vec<messages::Model>, i32), DbErr>;

            async fn mark_as_read(
                &self,
                message_id: Option<i32>,
                message_ids: Vec<i32>,
                from_user_id: Option<i32>,
                to_user_id: Option<i32>,
            ) -> Result<i32, DbErr>;

            async fn delete_message(&self, message_id: i32) -> Result<bool, DbErr>;
        }
    }

    fn create_test_message() -> messages::Model {
        let dt = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        
        messages::Model {
            id: 1,
            sender_id: 1,
            receiver_id: 2,
            content: "テストメッセージ".to_string(),
            is_read: false,
            created_at: dt,
            updated_at: dt,
            deleted_at: None,
        }
    }

    #[tokio::test]
    async fn test_send_message() {
        let mut mock_usecase = MockMessageUseCase::new();
        let test_message = create_test_message();
        
        mock_usecase
            .expect_send_message()
            .with(eq(1), eq(2), eq("こんにちは".to_string()))
            .returning(move |_, _, _| Ok(test_message.clone()));
        
        let handler = MessageHandler::new(mock_usecase);
        
        let request = Request::new(SendMessageRequest {
            sender_id: 1,
            receiver_id: 2,
            content: "こんにちは".to_string(),
        });
        
        let response = handler.send_message(request).await.unwrap();
        let result = response.into_inner();
        
        assert!(result.message.is_some());
        let message = result.message.unwrap();
        assert_eq!(message.id, 1);
        assert_eq!(message.sender_id, 1);
        assert_eq!(message.receiver_id, 2);
        assert_eq!(message.content, "テストメッセージ");
    }

    #[tokio::test]
    async fn test_list_messages() {
        let mut mock_usecase = MockMessageUseCase::new();
        let test_message = create_test_message();
        
        mock_usecase
            .expect_list_messages()
            .with(eq(1), eq(false), eq(1), eq(10))
            .returning(move |_, _, _, _| Ok((vec![test_message.clone()], 1, 0)));
        
        let handler = MessageHandler::new(mock_usecase);
        
        let request = Request::new(ListMessagesRequest {
            user_id: 1,
            unread_only: false,
            page: 1,
            per_page: 10,
        });
        
        let response = handler.list_messages(request).await.unwrap();
        let result = response.into_inner();
        
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.total_count, 1);
        assert_eq!(result.unread_count, 0);
    }

    #[tokio::test]
    async fn test_get_conversation() {
        let mut mock_usecase = MockMessageUseCase::new();
        let test_message = create_test_message();
        
        mock_usecase
            .expect_get_conversation()
            .with(eq(1), eq(2), eq(1), eq(10))
            .returning(move |_, _, _, _| Ok((vec![test_message.clone()], 1)));
        
        let handler = MessageHandler::new(mock_usecase);
        
        let request = Request::new(GetConversationRequest {
            user_id: 1,
            peer_id: 2,
            page: 1,
            per_page: 10,
        });
        
        let response = handler.get_conversation(request).await.unwrap();
        let result = response.into_inner();
        
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.total_count, 1);
    }

    #[tokio::test]
    async fn test_mark_as_read() {
        let mut mock_usecase = MockMessageUseCase::new();
        
        mock_usecase
            .expect_mark_as_read()
            .with(eq(Some(1)), eq(vec![]), eq(None), eq(None))
            .returning(|_, _, _, _| Ok(1));
        
        let handler = MessageHandler::new(mock_usecase);
        
        let request = Request::new(MarkAsReadRequest {
            message_id: 1,
            message_ids: vec![],
            from_user_id: None,
            to_user_id: None,
        });
        
        let response = handler.mark_as_read(request).await.unwrap();
        let result = response.into_inner();
        
        assert_eq!(result.updated_count, 1);
    }

    #[tokio::test]
    async fn test_delete_message() {
        let mut mock_usecase = MockMessageUseCase::new();
        
        mock_usecase
            .expect_delete_message()
            .with(eq(1))
            .returning(|_| Ok(true));
        
        let handler = MessageHandler::new(mock_usecase);
        
        let request = Request::new(DeleteMessageRequest {
            message_id: 1,
        });
        
        let response = handler.delete_message(request).await.unwrap();
        let result = response.into_inner();
        
        assert!(result.success);
    }
}
