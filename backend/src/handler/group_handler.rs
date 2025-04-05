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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{group_members, group_messages, groups, users};
    use crate::usecase::group_usecase::GroupUseCase;
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;
    // GroupUseCaseのモック
    mock! {
        pub GroupUseCase {}

        #[async_trait]
        impl GroupUseCase for GroupUseCase {
            async fn create_group(
                &self,
                name: String,
                description: Option<String>,
                creator_id: i32,
                initial_member_ids: Vec<i32>,
            ) -> Result<groups::Model, sea_orm::DbErr>;

            async fn get_group(
                &self,
                group_id: i32,
            ) -> Result<Option<groups::Model>, sea_orm::DbErr>;

            async fn list_groups(
                &self,
                user_id: i32,
                page: i32,
                per_page: i32,
            ) -> Result<(Vec<groups::Model>, i32), sea_orm::DbErr>;

            async fn update_group(
                &self,
                group_id: i32,
                name: Option<String>,
                description: Option<String>,
            ) -> Result<groups::Model, sea_orm::DbErr>;

            async fn delete_group(
                &self,
                group_id: i32,
            ) -> Result<bool, sea_orm::DbErr>;

            async fn add_group_member(
                &self,
                group_id: i32,
                user_id: i32,
                role: String,
            ) -> Result<group_members::Model, sea_orm::DbErr>;

            async fn remove_group_member(
                &self,
                group_id: i32,
                user_id: i32,
            ) -> Result<bool, sea_orm::DbErr>;

            async fn list_group_members(
                &self,
                group_id: i32,
                page: i32,
                per_page: i32,
            ) -> Result<(Vec<(group_members::Model, Option<users::Model>)>, i32), sea_orm::DbErr>;

            async fn send_group_message(
                &self,
                group_id: i32,
                sender_id: i32,
                content: String,
            ) -> Result<group_messages::Model, sea_orm::DbErr>;

            async fn list_group_messages(
                &self,
                group_id: i32,
                user_id: i32,
                page: i32,
                per_page: i32,
            ) -> Result<(Vec<(group_messages::Model, Vec<i32>)>, i32, i32), sea_orm::DbErr>;

            async fn mark_group_message_as_read(
                &self,
                group_id: i32,
                message_id: i32,
                user_id: i32,
            ) -> Result<bool, sea_orm::DbErr>;

            async fn delete_group_message(
                &self,
                message_id: i32,
            ) -> Result<bool, sea_orm::DbErr>;
        }
    }

    fn create_test_group() -> groups::Model {
        let dt = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        groups::Model {
            id: 1,
            name: "テストグループ".to_string(),
            description: Some("テスト説明".to_string()),
            creator_id: 1,
            created_at: dt,
            updated_at: dt,
            deleted_at: None,
        }
    }

    #[allow(dead_code)]
    fn create_test_group_member() -> group_members::Model {
        let dt = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        group_members::Model {
            id: 1,
            group_id: 1,
            user_id: 1,
            role: "admin".to_string(),
            created_at: dt,
            updated_at: dt,
            deleted_at: None,
        }
    }

    #[allow(dead_code)]
    fn create_test_user() -> users::Model {
        let dt = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        users::Model {
            id: 1,
            name: "テストユーザー".to_string(),
            email: "test@example.com".to_string(),
            description: Some("テスト説明".to_string()),
            age: Some(30),
            gender: Some("男性".to_string()),
            address: Some("東京都".to_string()),
            created_at: dt,
            updated_at: dt,
            deleted_at: None,
        }
    }

    #[allow(dead_code)]
    fn create_test_group_message() -> group_messages::Model {
        let dt = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        group_messages::Model {
            id: 1,
            group_id: 1,
            sender_id: 1,
            content: "テストメッセージ".to_string(),
            created_at: dt,
            updated_at: dt,
            deleted_at: None,
        }
    }

    #[tokio::test]
    async fn test_create_group() {
        let mut mock_usecase = MockGroupUseCase::new();
        let test_group = create_test_group();

        mock_usecase
            .expect_create_group()
            .with(
                eq("テストグループ".to_string()),
                eq(Some("テスト説明".to_string())),
                eq(1),
                eq(vec![1, 2, 3]),
            )
            .returning(|_, _, _, _| Ok(create_test_group()));

        let handler = GroupHandler::new(mock_usecase);
        let request = Request::new(CreateGroupRequest {
            name: "テストグループ".to_string(),
            description: "テスト説明".to_string(),
            creator_id: 1,
            initial_member_ids: vec![1, 2, 3],
        });

        let response = handler.create_group(request).await.unwrap();
        let group = response.into_inner().group.unwrap();

        assert_eq!(group.id, test_group.id as u64);
        assert_eq!(group.name, test_group.name);
        assert_eq!(group.description, test_group.description.unwrap());
        assert_eq!(group.creator_id, test_group.creator_id as u64);
    }

    #[tokio::test]
    async fn test_get_group() {
        let mut mock_usecase = MockGroupUseCase::new();
        let test_group = create_test_group();

        mock_usecase
            .expect_get_group()
            .with(eq(1))
            .returning(|_| Ok(Some(create_test_group())));

        let handler = GroupHandler::new(mock_usecase);
        let request = Request::new(GetGroupRequest { group_id: 1 });

        let response = handler.get_group(request).await.unwrap();
        let group = response.into_inner().group.unwrap();

        assert_eq!(group.id, test_group.id as u64);
        assert_eq!(group.name, test_group.name);
        assert_eq!(group.description, test_group.description.unwrap());
    }

    #[tokio::test]
    async fn test_get_group_not_found() {
        let mut mock_usecase = MockGroupUseCase::new();

        mock_usecase
            .expect_get_group()
            .with(eq(999))
            .returning(|_| Ok(None));

        let handler = GroupHandler::new(mock_usecase);
        let request = Request::new(GetGroupRequest { group_id: 999 });

        let result = handler.get_group(request).await;
        assert!(result.is_err());

        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn test_list_groups() {
        let mut mock_usecase = MockGroupUseCase::new();
        let test_groups = vec![create_test_group(), create_test_group()];

        mock_usecase
            .expect_list_groups()
            .with(eq(1), eq(1), eq(10))
            .returning(|_, _, _| Ok((vec![create_test_group(), create_test_group()], 2)));

        let handler = GroupHandler::new(mock_usecase);
        let request = Request::new(ListGroupsRequest {
            user_id: 1,
            page: 1,
            per_page: 10,
        });

        let response = handler.list_groups(request).await.unwrap();
        let groups = response.into_inner().groups;
        
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].id, test_groups[0].id as u64);
        assert_eq!(groups[0].name, test_groups[0].name);
    }

    #[tokio::test]
    async fn test_update_group() {
        let mut mock_usecase = MockGroupUseCase::new();
        let mut test_group = create_test_group();
        test_group.name = "更新グループ".to_string();
        test_group.description = Some("更新説明".to_string());

        mock_usecase
            .expect_update_group()
            .with(
                eq(1),
                eq(Some("更新グループ".to_string())),
                eq(Some("更新説明".to_string())),
            )
            .returning(|_, _, _| {
                let mut group = create_test_group();
                group.name = "更新グループ".to_string();
                group.description = Some("更新説明".to_string());
                Ok(group)
            });

        let handler = GroupHandler::new(mock_usecase);
        let request = Request::new(UpdateGroupRequest {
            group_id: 1,
            name: Some("更新グループ".to_string()),
            description: Some("更新説明".to_string()),
        });

        let response = handler.update_group(request).await.unwrap();
        let group = response.into_inner().group.unwrap();

        assert_eq!(group.id, test_group.id as u64);
        assert_eq!(group.name, "更新グループ");
        assert_eq!(group.description, "更新説明");
    }

    #[tokio::test]
    async fn test_delete_group() {
        let mut mock_usecase = MockGroupUseCase::new();

        mock_usecase
            .expect_delete_group()
            .with(eq(1))
            .returning(|_| Ok(true));

        let handler = GroupHandler::new(mock_usecase);
        let request = Request::new(DeleteGroupRequest { group_id: 1 });

        let response = handler.delete_group(request).await.unwrap();
        assert_eq!(response.into_inner().success, true);
    }
}
