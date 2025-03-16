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
