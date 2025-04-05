use crate::group_proto::group_service_server::GroupService;
use crate::group_proto::{
    AddGroupMemberRequest, AddGroupMemberResponse, CreateGroupRequest, CreateGroupResponse,
    DeleteGroupMessageRequest, DeleteGroupMessageResponse, DeleteGroupRequest, DeleteGroupResponse,
    GetGroupRequest, GetGroupResponse, Group, GroupMember, GroupMessage, ListGroupMembersRequest,
    ListGroupMembersResponse, ListGroupMessagesRequest, ListGroupMessagesResponse, ListGroupsRequest,
    ListGroupsResponse, MarkGroupMessageAsReadRequest, MarkGroupMessageAsReadResponse,
    RemoveGroupMemberRequest, RemoveGroupMemberResponse, SendGroupMessageRequest,
    SendGroupMessageResponse, UpdateGroupRequest, UpdateGroupResponse,
};
use crate::usecase::group_usecase::GroupUseCase;
use chrono::NaiveDateTime;
use tonic::{Request, Response, Status};

pub struct GroupHandler<U> {
    usecase: U,
}

impl<U: GroupUseCase> GroupHandler<U> {
    pub fn new(usecase: U) -> Self {
        Self { usecase }
    }

    // NaiveDateTime を文字列に変換するヘルパー関数
    fn format_datetime(dt: NaiveDateTime) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    // グループエンティティを Proto グループに変換するヘルパー関数
    fn to_proto_group(group: &crate::domain::entity::groups::Model) -> Group {
        Group {
            id: group.id as u64,
            name: group.name.clone(),
            description: group.description.clone().unwrap_or_default(),
            creator_id: group.creator_id as u64,
            created_at: Self::format_datetime(group.created_at),
            updated_at: Self::format_datetime(group.updated_at),
        }
    }

    // グループメンバーエンティティを Proto グループメンバーに変換するヘルパー関数
    fn to_proto_group_member(
        member: &crate::domain::entity::group_members::Model,
        user: &Option<crate::domain::entity::users::Model>,
    ) -> GroupMember {
        let (user_name, user_email) = if let Some(user) = user {
            (user.name.clone(), user.email.clone())
        } else {
            (String::new(), String::new())
        };

        GroupMember {
            id: member.id as u64,
            group_id: member.group_id as u64,
            user_id: member.user_id as u64,
            role: member.role.clone(),
            created_at: Self::format_datetime(member.created_at),
            updated_at: Self::format_datetime(member.updated_at),
            user_name,
            user_email,
        }
    }

    // グループメッセージエンティティを Proto グループメッセージに変換するヘルパー関数
    fn to_proto_group_message(
        message: &crate::domain::entity::group_messages::Model,
        read_by_user_ids: &[i32],
        sender_name: Option<String>,
    ) -> GroupMessage {
        GroupMessage {
            id: message.id as u64,
            group_id: message.group_id as u64,
            sender_id: message.sender_id as u64,
            content: message.content.clone(),
            created_at: Self::format_datetime(message.created_at),
            updated_at: Self::format_datetime(message.updated_at),
            sender_name: sender_name.unwrap_or_default(),
            read_by_user_ids: read_by_user_ids.iter().map(|&id| id as u64).collect(),
        }
    }
}

#[tonic::async_trait]
impl<U: GroupUseCase + Send + Sync + 'static> GroupService for GroupHandler<U> {
    async fn create_group(
        &self,
        request: Request<CreateGroupRequest>,
    ) -> Result<Response<CreateGroupResponse>, Status> {
        let req = request.into_inner();
        let initial_member_ids = req
            .initial_member_ids
            .iter()
            .map(|&id| id as i32)
            .collect::<Vec<i32>>();

        let group = self
            .usecase
            .create_group(
                req.name,
                if req.description.is_empty() {
                    None
                } else {
                    Some(req.description)
                },
                req.creator_id as i32,
                initial_member_ids,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateGroupResponse {
            group: Some(Self::to_proto_group(&group)),
        }))
    }

    async fn get_group(
        &self,
        request: Request<GetGroupRequest>,
    ) -> Result<Response<GetGroupResponse>, Status> {
        let req = request.into_inner();
        let group = self
            .usecase
            .get_group(req.group_id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("グループが見つかりません"))?;

        Ok(Response::new(GetGroupResponse {
            group: Some(Self::to_proto_group(&group)),
        }))
    }

    async fn list_groups(
        &self,
        request: Request<ListGroupsRequest>,
    ) -> Result<Response<ListGroupsResponse>, Status> {
        let req = request.into_inner();
        let (groups, total_count) = self
            .usecase
            .list_groups(req.user_id as i32, req.page, req.per_page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_groups = groups
            .iter()
            .map(|g| Self::to_proto_group(g))
            .collect();

        Ok(Response::new(ListGroupsResponse {
            groups: proto_groups,
            total_count,
        }))
    }

    async fn update_group(
        &self,
        request: Request<UpdateGroupRequest>,
    ) -> Result<Response<UpdateGroupResponse>, Status> {
        let req = request.into_inner();
        let name = req.name.map(|n| n);
        let description = req.description.map(|d| d);

        let group = self
            .usecase
            .update_group(req.group_id as i32, name, description)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateGroupResponse {
            group: Some(Self::to_proto_group(&group)),
        }))
    }

    async fn delete_group(
        &self,
        request: Request<DeleteGroupRequest>,
    ) -> Result<Response<DeleteGroupResponse>, Status> {
        let req = request.into_inner();
        let success = self
            .usecase
            .delete_group(req.group_id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteGroupResponse { success }))
    }

    async fn add_group_member(
        &self,
        request: Request<AddGroupMemberRequest>,
    ) -> Result<Response<AddGroupMemberResponse>, Status> {
        let req = request.into_inner();
        let member = self
            .usecase
            .add_group_member(req.group_id as i32, req.user_id as i32, req.role)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // ユーザー情報は別途取得する必要があるが、ここでは簡略化のためNoneを使用
        let proto_member = Self::to_proto_group_member(&member, &None);

        Ok(Response::new(AddGroupMemberResponse {
            member: Some(proto_member),
        }))
    }

    async fn remove_group_member(
        &self,
        request: Request<RemoveGroupMemberRequest>,
    ) -> Result<Response<RemoveGroupMemberResponse>, Status> {
        let req = request.into_inner();
        let success = self
            .usecase
            .remove_group_member(req.group_id as i32, req.user_id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RemoveGroupMemberResponse { success }))
    }

    async fn list_group_members(
        &self,
        request: Request<ListGroupMembersRequest>,
    ) -> Result<Response<ListGroupMembersResponse>, Status> {
        let req = request.into_inner();
        let (members, total_count) = self
            .usecase
            .list_group_members(req.group_id as i32, req.page, req.per_page)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_members = members
            .iter()
            .map(|(m, u)| Self::to_proto_group_member(m, u))
            .collect();

        Ok(Response::new(ListGroupMembersResponse {
            members: proto_members,
            total_count,
        }))
    }

    async fn send_group_message(
        &self,
        request: Request<SendGroupMessageRequest>,
    ) -> Result<Response<SendGroupMessageResponse>, Status> {
        let req = request.into_inner();
        let message = self
            .usecase
            .send_group_message(req.group_id as i32, req.sender_id as i32, req.content)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 送信者の名前は別途取得する必要があるが、ここでは簡略化のためNoneを使用
        let proto_message = Self::to_proto_group_message(&message, &[req.sender_id as i32], None);

        Ok(Response::new(SendGroupMessageResponse {
            message: Some(proto_message),
        }))
    }

    async fn list_group_messages(
        &self,
        request: Request<ListGroupMessagesRequest>,
    ) -> Result<Response<ListGroupMessagesResponse>, Status> {
        let req = request.into_inner();
        let (messages, total_count, unread_count) = self
            .usecase
            .list_group_messages(
                req.group_id as i32,
                req.user_id as i32,
                req.page,
                req.per_page,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 送信者の名前を取得するためのマップを作成する必要があるが、ここでは簡略化
        let proto_messages = messages
            .iter()
            .map(|(m, read_by)| Self::to_proto_group_message(m, read_by, None))
            .collect();

        Ok(Response::new(ListGroupMessagesResponse {
            messages: proto_messages,
            total_count,
            unread_count,
        }))
    }

    async fn mark_group_message_as_read(
        &self,
        request: Request<MarkGroupMessageAsReadRequest>,
    ) -> Result<Response<MarkGroupMessageAsReadResponse>, Status> {
        let req = request.into_inner();
        let success = self
            .usecase
            .mark_group_message_as_read(
                req.group_id as i32,
                req.message_id as i32,
                req.user_id as i32,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(MarkGroupMessageAsReadResponse { success }))
    }

    async fn delete_group_message(
        &self,
        request: Request<DeleteGroupMessageRequest>,
    ) -> Result<Response<DeleteGroupMessageResponse>, Status> {
        let req = request.into_inner();
        let success = self
            .usecase
            .delete_group_message(req.message_id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteGroupMessageResponse { success }))
    }
}
