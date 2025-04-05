use crate::domain::entity::{group_members, group_messages, groups};
use crate::domain::repository::group::GroupRepository;
use crate::handler::group_handler::GroupHandler;
use crate::usecase::group_usecase::{GroupUseCase, GroupUseCaseImpl};
use async_trait::async_trait;
use mockall::predicate::*;
use mockall::*;
use sea_orm::DbErr;
use std::sync::Arc;

// GroupRepositoryのモック
mock! {
    pub GroupRepo {}

    #[async_trait]
    impl GroupRepository for GroupRepo {
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
}

// テスト用のヘルパー関数
fn create_test_group() -> groups::Model {
    groups::Model {
        id: 1,
        name: "テストグループ".to_string(),
        description: Some("テスト用のグループです".to_string()),
        creator_id: 1,
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        deleted_at: None,
    }
}

fn create_test_group_member() -> group_members::Model {
    group_members::Model {
        id: 1,
        group_id: 1,
        user_id: 2,
        role: "member".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        deleted_at: None,
    }
}

fn create_test_group_message() -> group_messages::Model {
    group_messages::Model {
        id: 1,
        group_id: 1,
        sender_id: 1,
        content: "テストメッセージ".to_string(),
        created_at: chrono::Utc::now().naive_utc(),
        updated_at: chrono::Utc::now().naive_utc(),
        deleted_at: None,
    }
}

#[tokio::test]
async fn test_create_group() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_create_group()
        .with(
            eq("テストグループ".to_string()),
            eq(Some("テスト用のグループです".to_string())),
            eq(1),
            eq(vec![2, 3]),
        )
        .returning(|_, _, _, _| Ok(create_test_group()));

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::CreateGroupRequest {
        name: "テストグループ".to_string(),
        description: "テスト用のグループです".to_string(),
        creator_id: 1,
        initial_member_ids: vec![2, 3],
    });

    // ハンドラーの呼び出し
    let response = handler.create_group(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert!(response.group.is_some());
    let group = response.group.unwrap();
    assert_eq!(group.id, 1);
    assert_eq!(group.name, "テストグループ");
    assert_eq!(group.description, "テスト用のグループです");
    assert_eq!(group.creator_id, 1);
}

#[tokio::test]
async fn test_get_group() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_get_group()
        .with(eq(1))
        .returning(|_| Ok(Some(create_test_group())));

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::GetGroupRequest { group_id: 1 });

    // ハンドラーの呼び出し
    let response = handler.get_group(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert!(response.group.is_some());
    let group = response.group.unwrap();
    assert_eq!(group.id, 1);
    assert_eq!(group.name, "テストグループ");
}

#[tokio::test]
async fn test_list_groups() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_list_groups()
        .with(eq(1), eq(1), eq(10))
        .returning(|_, _, _| {
            let groups = vec![create_test_group()];
            Ok((groups, 1))
        });

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::ListGroupsRequest {
        user_id: 1,
        page: 1,
        per_page: 10,
    });

    // ハンドラーの呼び出し
    let response = handler.list_groups(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert_eq!(response.groups.len(), 1);
    assert_eq!(response.total_count, 1);
    assert_eq!(response.groups[0].id, 1);
    assert_eq!(response.groups[0].name, "テストグループ");
}

#[tokio::test]
async fn test_update_group() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_update_group()
        .with(
            eq(1),
            eq(Some("更新されたグループ名".to_string())),
            eq(Some("更新された説明".to_string())),
        )
        .returning(|_, _, _| {
            let mut group = create_test_group();
            group.name = "更新されたグループ名".to_string();
            group.description = Some("更新された説明".to_string());
            Ok(group)
        });

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::UpdateGroupRequest {
        group_id: 1,
        name: Some(prost_types::StringValue {
            value: "更新されたグループ名".to_string(),
        }),
        description: Some(prost_types::StringValue {
            value: "更新された説明".to_string(),
        }),
    });

    // ハンドラーの呼び出し
    let response = handler.update_group(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert!(response.group.is_some());
    let group = response.group.unwrap();
    assert_eq!(group.id, 1);
    assert_eq!(group.name, "更新されたグループ名");
    assert_eq!(group.description, "更新された説明");
}

#[tokio::test]
async fn test_delete_group() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_delete_group()
        .with(eq(1))
        .returning(|_| Ok(true));

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::DeleteGroupRequest { group_id: 1 });

    // ハンドラーの呼び出し
    let response = handler.delete_group(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert!(response.success);
}

#[tokio::test]
async fn test_add_group_member() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_add_group_member()
        .with(eq(1), eq(2), eq("member".to_string()))
        .returning(|_, _, _| Ok(create_test_group_member()));

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::AddGroupMemberRequest {
        group_id: 1,
        user_id: 2,
        role: "member".to_string(),
    });

    // ハンドラーの呼び出し
    let response = handler.add_group_member(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert!(response.member.is_some());
    let member = response.member.unwrap();
    assert_eq!(member.group_id, 1);
    assert_eq!(member.user_id, 2);
    assert_eq!(member.role, "member");
}

#[tokio::test]
async fn test_remove_group_member() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_remove_group_member()
        .with(eq(1), eq(2))
        .returning(|_, _| Ok(true));

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::RemoveGroupMemberRequest {
        group_id: 1,
        user_id: 2,
    });

    // ハンドラーの呼び出し
    let response = handler.remove_group_member(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert!(response.success);
}

#[tokio::test]
async fn test_list_group_members() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_list_group_members()
        .with(eq(1), eq(1), eq(10))
        .returning(|_, _, _| {
            let members = vec![(create_test_group_member(), None)];
            Ok((members, 1))
        });

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::ListGroupMembersRequest {
        group_id: 1,
        page: 1,
        per_page: 10,
    });

    // ハンドラーの呼び出し
    let response = handler.list_group_members(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert_eq!(response.members.len(), 1);
    assert_eq!(response.total_count, 1);
    assert_eq!(response.members[0].group_id, 1);
    assert_eq!(response.members[0].user_id, 2);
    assert_eq!(response.members[0].role, "member");
}

#[tokio::test]
async fn test_send_group_message() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_send_group_message()
        .with(eq(1), eq(1), eq("テストメッセージ".to_string()))
        .returning(|_, _, _| Ok(create_test_group_message()));

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::SendGroupMessageRequest {
        group_id: 1,
        sender_id: 1,
        content: "テストメッセージ".to_string(),
    });

    // ハンドラーの呼び出し
    let response = handler.send_group_message(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert!(response.message.is_some());
    let message = response.message.unwrap();
    assert_eq!(message.group_id, 1);
    assert_eq!(message.sender_id, 1);
    assert_eq!(message.content, "テストメッセージ");
}

#[tokio::test]
async fn test_list_group_messages() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_list_group_messages()
        .with(eq(1), eq(2), eq(1), eq(10))
        .returning(|_, _, _, _| {
            let messages = vec![(create_test_group_message(), vec![1])];
            Ok((messages, 1, 0))
        });

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::ListGroupMessagesRequest {
        group_id: 1,
        user_id: 2,
        page: 1,
        per_page: 10,
    });

    // ハンドラーの呼び出し
    let response = handler.list_group_messages(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert_eq!(response.messages.len(), 1);
    assert_eq!(response.total_count, 1);
    assert_eq!(response.unread_count, 0);
    assert_eq!(response.messages[0].group_id, 1);
    assert_eq!(response.messages[0].sender_id, 1);
    assert_eq!(response.messages[0].content, "テストメッセージ");
    assert_eq!(response.messages[0].read_by_user_ids, vec![1]);
}

#[tokio::test]
async fn test_mark_group_message_as_read() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_mark_group_message_as_read()
        .with(eq(1), eq(1), eq(2))
        .returning(|_, _, _| Ok(true));

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::MarkGroupMessageAsReadRequest {
        group_id: 1,
        message_id: 1,
        user_id: 2,
    });

    // ハンドラーの呼び出し
    let response = handler.mark_group_message_as_read(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert!(response.success);
}

#[tokio::test]
async fn test_delete_group_message() {
    // モックの設定
    let mut mock_repo = MockGroupRepo::new();
    mock_repo
        .expect_delete_group_message()
        .with(eq(1))
        .returning(|_| Ok(true));

    // ユースケースとハンドラーの作成
    let usecase = GroupUseCaseImpl::new(mock_repo);
    let handler = GroupHandler::new(usecase);

    // リクエストの作成
    let request = tonic::Request::new(crate::group_proto::DeleteGroupMessageRequest {
        message_id: 1,
    });

    // ハンドラーの呼び出し
    let response = handler.delete_group_message(request).await.unwrap();
    let response = response.into_inner();

    // 検証
    assert!(response.success);
}
