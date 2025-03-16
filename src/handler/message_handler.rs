use crate::message_proto::message_service_server::MessageService;
use crate::message_proto::{
    DeleteMessageRequest, DeleteMessageResponse, GetConversationRequest, GetConversationResponse,
    ListMessagesRequest, ListMessagesResponse, MarkAsReadRequest, MarkAsReadResponse, Message,
    SendMessageRequest, SendMessageResponse,
};
use crate::usecase::message_usecase::MessageUseCase;
use chrono::NaiveDateTime;
use tonic::{Request, Response, Status};

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
