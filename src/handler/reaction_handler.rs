use crate::reaction_proto::reaction_service_server::ReactionService;
use crate::reaction_proto::{
    AddReactionRequest, AddReactionResponse, CountReactionsByTypeRequest, CountReactionsByTypeResponse,
    GetReactionTypeRequest, GetReactionTypeResponse, GetReactionsForMessageRequest,
    GetReactionsForMessageResponse, ListReactionTypesRequest, ListReactionTypesResponse, Reaction,
    ReactionType, ReactionTypeCount, RemoveReactionRequest, RemoveReactionResponse,
};
use crate::usecase::reaction_usecase::ReactionUseCase;
use chrono::NaiveDateTime;
use tonic::{Request, Response, Status};

pub struct ReactionHandler<U> {
    usecase: U,
}

impl<U: ReactionUseCase> ReactionHandler<U> {
    pub fn new(usecase: U) -> Self {
        Self { usecase }
    }

    // NaiveDateTime を文字列に変換するヘルパー関数
    fn format_datetime(dt: NaiveDateTime) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    // リアクションエンティティを Proto リアクションに変換するヘルパー関数
    fn to_proto_reaction(reaction: &crate::domain::entity::reactions::Model) -> Reaction {
        Reaction {
            id: reaction.id as u64,
            user_id: reaction.user_id as u64,
            message_id: reaction.message_id as u64,
            reaction_type_id: reaction.reaction_type_id as u64,
            created_at: Self::format_datetime(reaction.created_at),
            updated_at: Self::format_datetime(reaction.updated_at),
        }
    }

    // リアクションタイプエンティティを Proto リアクションタイプに変換するヘルパー関数
    fn to_proto_reaction_type(
        reaction_type: &crate::domain::entity::reaction_types::Model,
    ) -> ReactionType {
        ReactionType {
            id: reaction_type.id as u64,
            name: reaction_type.name.clone(),
            emoji: reaction_type.emoji.clone(),
        }
    }
}

#[tonic::async_trait]
impl<U: ReactionUseCase + Send + Sync + 'static> ReactionService for ReactionHandler<U> {
    async fn add_reaction(
        &self,
        request: Request<AddReactionRequest>,
    ) -> Result<Response<AddReactionResponse>, Status> {
        let req = request.into_inner();
        let reaction = self
            .usecase
            .add_reaction(
                req.user_id as i32,
                req.message_id as i32,
                req.reaction_type_id as i32,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AddReactionResponse {
            reaction: Some(Self::to_proto_reaction(&reaction)),
        }))
    }

    async fn remove_reaction(
        &self,
        request: Request<RemoveReactionRequest>,
    ) -> Result<Response<RemoveReactionResponse>, Status> {
        let req = request.into_inner();
        let reaction_type_id = req.reaction_type_id.map(|id| id as i32);
        let removed_count = self
            .usecase
            .remove_reaction(req.user_id as i32, req.message_id as i32, reaction_type_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RemoveReactionResponse { removed_count }))
    }

    async fn get_reactions_for_message(
        &self,
        request: Request<GetReactionsForMessageRequest>,
    ) -> Result<Response<GetReactionsForMessageResponse>, Status> {
        let req = request.into_inner();
        let reactions = self
            .usecase
            .get_reactions_for_message(req.message_id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_reactions = reactions
            .iter()
            .map(|r| Self::to_proto_reaction(r))
            .collect();

        Ok(Response::new(GetReactionsForMessageResponse {
            reactions: proto_reactions,
        }))
    }

    async fn count_reactions_by_type(
        &self,
        request: Request<CountReactionsByTypeRequest>,
    ) -> Result<Response<CountReactionsByTypeResponse>, Status> {
        let req = request.into_inner();
        let counts = self
            .usecase
            .count_reactions_by_type(req.message_id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_counts = counts
            .iter()
            .map(|(reaction_type, count)| ReactionTypeCount {
                reaction_type: Some(Self::to_proto_reaction_type(reaction_type)),
                count: *count,
            })
            .collect();

        Ok(Response::new(CountReactionsByTypeResponse {
            counts: proto_counts,
        }))
    }

    async fn list_reaction_types(
        &self,
        _request: Request<ListReactionTypesRequest>,
    ) -> Result<Response<ListReactionTypesResponse>, Status> {
        let reaction_types = self
            .usecase
            .list_reaction_types()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_reaction_types = reaction_types
            .iter()
            .map(|rt| Self::to_proto_reaction_type(rt))
            .collect();

        Ok(Response::new(ListReactionTypesResponse {
            reaction_types: proto_reaction_types,
        }))
    }

    async fn get_reaction_type(
        &self,
        request: Request<GetReactionTypeRequest>,
    ) -> Result<Response<GetReactionTypeResponse>, Status> {
        let req = request.into_inner();
        let reaction_type = self
            .usecase
            .get_reaction_type(req.id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Reaction type not found"))?;

        Ok(Response::new(GetReactionTypeResponse {
            reaction_type: Some(Self::to_proto_reaction_type(&reaction_type)),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{reactions, reaction_types};
    use crate::usecase::reaction_usecase::ReactionUseCase;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use mockall::predicate::*;
    use mockall::*;
    use sea_orm::DbErr;

    // ReactionUseCaseのモック
    mock! {
        pub ReactionUseCase {}

        #[async_trait]
        impl ReactionUseCase for ReactionUseCase {
            async fn add_reaction(
                &self,
                user_id: i32,
                message_id: i32,
                reaction_type_id: i32,
            ) -> Result<reactions::Model, DbErr>;

            async fn remove_reaction(
                &self,
                user_id: i32,
                message_id: i32,
                reaction_type_id: Option<i32>,
            ) -> Result<i32, DbErr>;

            async fn get_reactions_for_message(
                &self,
                message_id: i32,
            ) -> Result<Vec<reactions::Model>, DbErr>;

            async fn count_reactions_by_type(
                &self,
                message_id: i32,
            ) -> Result<Vec<(reaction_types::Model, i64)>, DbErr>;

            async fn list_reaction_types(
                &self,
            ) -> Result<Vec<reaction_types::Model>, DbErr>;

            async fn get_reaction_type(
                &self,
                id: i32,
            ) -> Result<Option<reaction_types::Model>, DbErr>;
        }
    }

    fn create_test_reaction() -> reactions::Model {
        let dt = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        
        reactions::Model {
            id: 1,
            user_id: 1,
            message_id: 1,
            reaction_type_id: 1,
            created_at: dt,
            updated_at: dt,
            deleted_at: None,
        }
    }

    fn create_test_reaction_type() -> reaction_types::Model {
        let dt = NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
            
        reaction_types::Model {
            id: 1,
            name: "いいね".to_string(),
            emoji: "👍".to_string(),
            created_at: dt,
            updated_at: dt,
        }
    }

    #[tokio::test]
    async fn test_add_reaction() {
        let mut mock_usecase = MockReactionUseCase::new();
        let test_reaction = create_test_reaction();
        
        mock_usecase
            .expect_add_reaction()
            .with(eq(1), eq(1), eq(1))
            .returning(|_, _, _| Ok(create_test_reaction()));
        
        let handler = ReactionHandler::new(mock_usecase);
        let request = Request::new(AddReactionRequest {
            user_id: 1,
            message_id: 1,
            reaction_type_id: 1,
        });
        
        let response = handler.add_reaction(request).await.unwrap();
        let reaction = response.into_inner().reaction.unwrap();
        
        assert_eq!(reaction.id, test_reaction.id as u64);
        assert_eq!(reaction.user_id, test_reaction.user_id as u64);
        assert_eq!(reaction.message_id, test_reaction.message_id as u64);
        assert_eq!(reaction.reaction_type_id, test_reaction.reaction_type_id as u64);
    }

    #[tokio::test]
    async fn test_remove_reaction() {
        let mut mock_usecase = MockReactionUseCase::new();
        
        mock_usecase
            .expect_remove_reaction()
            .with(eq(1), eq(1), eq(Some(1)))
            .returning(|_, _, _| Ok(1));
        
        let handler = ReactionHandler::new(mock_usecase);
        let request = Request::new(RemoveReactionRequest {
            user_id: 1,
            message_id: 1,
            reaction_type_id: Some(1),
        });
        
        let response = handler.remove_reaction(request).await.unwrap();
        let removed_count = response.into_inner().removed_count;
        
        assert_eq!(removed_count, 1);
    }

    #[tokio::test]
    async fn test_get_reactions_for_message() {
        let mut mock_usecase = MockReactionUseCase::new();
        let test_reactions = vec![create_test_reaction()];
        
        mock_usecase
            .expect_get_reactions_for_message()
            .with(eq(1))
            .returning(|_| Ok(vec![create_test_reaction()]));
        
        let handler = ReactionHandler::new(mock_usecase);
        let request = Request::new(GetReactionsForMessageRequest {
            message_id: 1,
        });
        
        let response = handler.get_reactions_for_message(request).await.unwrap();
        let reactions = response.into_inner().reactions;
        
        assert_eq!(reactions.len(), 1);
        assert_eq!(reactions[0].id, test_reactions[0].id as u64);
        assert_eq!(reactions[0].user_id, test_reactions[0].user_id as u64);
        assert_eq!(reactions[0].message_id, test_reactions[0].message_id as u64);
    }

    #[tokio::test]
    async fn test_count_reactions_by_type() {
        let mut mock_usecase = MockReactionUseCase::new();
        let test_reaction_type = create_test_reaction_type();
        
        mock_usecase
            .expect_count_reactions_by_type()
            .with(eq(1))
            .returning(|_| Ok(vec![(create_test_reaction_type(), 5i64)]));
        
        let handler = ReactionHandler::new(mock_usecase);
        let request = Request::new(CountReactionsByTypeRequest {
            message_id: 1,
        });
        
        let response = handler.count_reactions_by_type(request).await.unwrap();
        let counts = response.into_inner().counts;
        
        assert_eq!(counts.len(), 1);
        assert_eq!(counts[0].count, 5);
        let reaction_type = counts[0].reaction_type.as_ref().unwrap();
        assert_eq!(reaction_type.id, test_reaction_type.id as u64);
        assert_eq!(reaction_type.name, test_reaction_type.name);
        assert_eq!(reaction_type.emoji, test_reaction_type.emoji);
    }

    #[tokio::test]
    async fn test_list_reaction_types() {
        let mut mock_usecase = MockReactionUseCase::new();
        let test_reaction_types = vec![create_test_reaction_type()];
        
        mock_usecase
            .expect_list_reaction_types()
            .returning(|| Ok(vec![create_test_reaction_type()]));
        
        let handler = ReactionHandler::new(mock_usecase);
        let request = Request::new(ListReactionTypesRequest {});
        
        let response = handler.list_reaction_types(request).await.unwrap();
        let reaction_types = response.into_inner().reaction_types;
        
        assert_eq!(reaction_types.len(), 1);
        assert_eq!(reaction_types[0].id, test_reaction_types[0].id as u64);
        assert_eq!(reaction_types[0].name, test_reaction_types[0].name);
        assert_eq!(reaction_types[0].emoji, test_reaction_types[0].emoji);
    }

    #[tokio::test]
    async fn test_get_reaction_type() {
        let mut mock_usecase = MockReactionUseCase::new();
        let test_reaction_type = create_test_reaction_type();
        
        mock_usecase
            .expect_get_reaction_type()
            .with(eq(1))
            .returning(|_| Ok(Some(create_test_reaction_type())));
        
        let handler = ReactionHandler::new(mock_usecase);
        let request = Request::new(GetReactionTypeRequest { id: 1 });
        
        let response = handler.get_reaction_type(request).await.unwrap();
        let reaction_type = response.into_inner().reaction_type.unwrap();
        
        assert_eq!(reaction_type.id, test_reaction_type.id as u64);
        assert_eq!(reaction_type.name, test_reaction_type.name);
        assert_eq!(reaction_type.emoji, test_reaction_type.emoji);
    }

    #[tokio::test]
    async fn test_get_reaction_type_not_found() {
        let mut mock_usecase = MockReactionUseCase::new();
        
        mock_usecase
            .expect_get_reaction_type()
            .with(eq(999))
            .returning(|_| Ok(None));
        
        let handler = ReactionHandler::new(mock_usecase);
        let request = Request::new(GetReactionTypeRequest { id: 999 });
        
        let result = handler.get_reaction_type(request).await;
        assert!(result.is_err());
        
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }
}
