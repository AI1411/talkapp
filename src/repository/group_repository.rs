use crate::domain::entity::{group_members, group_message_reads, group_messages, groups, users};
use crate::domain::repository::group::GroupRepository;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, NotSet, QueryOrder, Set, TransactionTrait};

pub struct PgGroupRepository {
    db: DatabaseConnection,
}

impl PgGroupRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl GroupRepository for PgGroupRepository {
    async fn create_group(
        &self,
        name: String,
        description: Option<String>,
        creator_id: i32,
        initial_member_ids: Vec<i32>,
    ) -> Result<groups::Model, DbErr> {
        let now = Utc::now().naive_utc();
        let new_group = groups::ActiveModel {
            id: NotSet,
            name: Set(name),
            description: Set(description),
            creator_id: Set(creator_id),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };

        // トランザクション開始
        let txn = self.db.begin().await?;

        // グループを作成
        let res = groups::Entity::insert(new_group).exec(&txn).await?;
        let group_id = res.last_insert_id;

        // 作成者を管理者として追加
        let creator_member = group_members::ActiveModel {
            id: NotSet,
            group_id: Set(group_id),
            user_id: Set(creator_id),
            role: Set("admin".to_string()),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };
        group_members::Entity::insert(creator_member)
            .exec(&txn)
            .await?;

        // 初期メンバーを追加（作成者以外）
        for user_id in initial_member_ids {
            if user_id != creator_id {
                let member = group_members::ActiveModel {
                    id: NotSet,
                    group_id: Set(group_id),
                    user_id: Set(user_id),
                    role: Set("member".to_string()),
                    created_at: Set(now),
                    updated_at: Set(now),
                    deleted_at: NotSet,
                };
                group_members::Entity::insert(member).exec(&txn).await?;
            }
        }

        // トランザクションをコミット
        txn.commit().await?;

        // 作成したグループを取得して返す
        let group = groups::Entity::find_by_id(group_id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("グループの作成に失敗しました".into()))?;

        Ok(group)
    }

    async fn get_group(&self, group_id: i32) -> Result<Option<groups::Model>, DbErr> {
        groups::Entity::find_by_id(group_id)
            .filter(groups::Column::DeletedAt.is_null())
            .one(&self.db)
            .await
    }

    async fn list_groups(
        &self,
        user_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<groups::Model>, i32), DbErr> {
        // ユーザーが所属するグループのIDを取得
        let group_ids = group_members::Entity::find()
            .filter(group_members::Column::UserId.eq(user_id))
            .filter(group_members::Column::DeletedAt.is_null())
            .all(&self.db)
            .await?
            .into_iter()
            .map(|m| m.group_id)
            .collect::<Vec<i32>>();

        if group_ids.is_empty() {
            return Ok((vec![], 0));
        }

        // グループ情報を取得
        let query = groups::Entity::find()
            .filter(groups::Column::Id.is_in(group_ids))
            .filter(groups::Column::DeletedAt.is_null())
            .order_by_desc(groups::Column::UpdatedAt);

        let total_count = query.clone().count(&self.db).await?;
        
        // ページネーションの計算（0ベースのページ番号に変換）
        let page_num = if page <= 0 { 0 } else { (page - 1) as u64 };
        let groups = query
            .paginate(&self.db, per_page as u64)
            .fetch_page(page_num)
            .await?;

        Ok((groups, total_count as i32))
    }

    async fn update_group(
        &self,
        group_id: i32,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<groups::Model, DbErr> {
        let now = Utc::now().naive_utc();
        let mut group: groups::ActiveModel = groups::Entity::find_by_id(group_id)
            .filter(groups::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("グループが見つかりません".into()))?
            .into();

        if let Some(name) = name {
            group.name = Set(name);
        }

        if let Some(description) = description {
            group.description = Set(Some(description));
        }

        group.updated_at = Set(now);

        let updated_group = group.update(&self.db).await?;
        Ok(updated_group)
    }

    async fn delete_group(&self, group_id: i32) -> Result<bool, DbErr> {
        let now = Utc::now().naive_utc();
        let result = groups::Entity::update_many()
            .filter(groups::Column::Id.eq(group_id))
            .filter(groups::Column::DeletedAt.is_null())
            .col_expr(groups::Column::DeletedAt, Expr::value(Some(now)))
            .col_expr(groups::Column::UpdatedAt, Expr::value(now))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected > 0)
    }

    async fn add_group_member(
        &self,
        group_id: i32,
        user_id: i32,
        role: String,
    ) -> Result<group_members::Model, DbErr> {
        // グループが存在するか確認
        let _group = groups::Entity::find_by_id(group_id)
            .filter(groups::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("グループが見つかりません".into()))?;

        // ユーザーが既にメンバーかどうか確認
        let existing_member = group_members::Entity::find()
            .filter(group_members::Column::GroupId.eq(group_id))
            .filter(group_members::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;

        if let Some(member) = existing_member {
            if member.deleted_at.is_none() {
                return Err(DbErr::Custom("ユーザーは既にグループのメンバーです".into()));
            }

            // 論理削除されていた場合は復活させる
            let now = Utc::now().naive_utc();
            let mut member: group_members::ActiveModel = member.into();
            member.deleted_at = Set(None);
            member.updated_at = Set(now);
            member.role = Set(role);
            let updated_member = member.update(&self.db).await?;
            return Ok(updated_member);
        }

        // 新しいメンバーを追加
        let now = Utc::now().naive_utc();
        let new_member = group_members::ActiveModel {
            id: NotSet,
            group_id: Set(group_id),
            user_id: Set(user_id),
            role: Set(role),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };

        let res = group_members::Entity::insert(new_member)
            .exec(&self.db)
            .await?;
        let member = group_members::Entity::find_by_id(res.last_insert_id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("メンバーの追加に失敗しました".into()))?;

        Ok(member)
    }

    async fn remove_group_member(&self, group_id: i32, user_id: i32) -> Result<bool, DbErr> {
        let now = Utc::now().naive_utc();
        let result = group_members::Entity::update_many()
            .filter(group_members::Column::GroupId.eq(group_id))
            .filter(group_members::Column::UserId.eq(user_id))
            .filter(group_members::Column::DeletedAt.is_null())
            .col_expr(group_members::Column::DeletedAt, Expr::value(Some(now)))
            .col_expr(group_members::Column::UpdatedAt, Expr::value(now))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected > 0)
    }

    async fn list_group_members(
        &self,
        group_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<(group_members::Model, Option<users::Model>)>, i32), DbErr> {
        // グループが存在するか確認
        let _group = groups::Entity::find_by_id(group_id)
            .filter(groups::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("グループが見つかりません".into()))?;

        // メンバー情報を取得（ユーザー情報も含む）
        let query = group_members::Entity::find()
            .filter(group_members::Column::GroupId.eq(group_id))
            .filter(group_members::Column::DeletedAt.is_null())
            .find_also_related(users::Entity)
            .order_by_asc(group_members::Column::Id);

        let total_count = group_members::Entity::find()
            .filter(group_members::Column::GroupId.eq(group_id))
            .filter(group_members::Column::DeletedAt.is_null())
            .count(&self.db)
            .await?;

        // ページネーションの計算（0ベースのページ番号に変換）
        let page_num = if page <= 0 { 0 } else { (page - 1) as u64 };
        let members = query
            .paginate(&self.db, per_page as u64)
            .fetch_page(page_num)
            .await?;

        Ok((members, total_count as i32))
    }

    async fn send_group_message(
        &self,
        group_id: i32,
        sender_id: i32,
        content: String,
    ) -> Result<group_messages::Model, DbErr> {
        // グループが存在するか確認
        let _group = groups::Entity::find_by_id(group_id)
            .filter(groups::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("グループが見つかりません".into()))?;

        // 送信者がグループのメンバーかどうか確認
        let is_member = group_members::Entity::find()
            .filter(group_members::Column::GroupId.eq(group_id))
            .filter(group_members::Column::UserId.eq(sender_id))
            .filter(group_members::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .is_some();

        if !is_member {
            return Err(DbErr::Custom(
                "送信者はグループのメンバーではありません".into(),
            ));
        }

        // メッセージを作成
        let now = Utc::now().naive_utc();
        let new_message = group_messages::ActiveModel {
            id: NotSet,
            group_id: Set(group_id),
            sender_id: Set(sender_id),
            content: Set(content),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };

        let res = group_messages::Entity::insert(new_message)
            .exec(&self.db)
            .await?;
        let message = group_messages::Entity::find_by_id(res.last_insert_id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("メッセージの送信に失敗しました".into()))?;

        // 送信者のメッセージを自動的に既読にする
        let _read = group_message_reads::ActiveModel {
            id: NotSet,
            group_id: Set(group_id),
            message_id: Set(message.id),
            user_id: Set(sender_id),
            read_at: Set(now),
        };

        let _ = group_message_reads::Entity::insert(_read)
            .exec(&self.db)
            .await?;

        Ok(message)
    }

    async fn list_group_messages(
        &self,
        group_id: i32,
        user_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<(group_messages::Model, Vec<i32>)>, i32, i32), DbErr> {
        // グループが存在するか確認
        let _group = groups::Entity::find_by_id(group_id)
            .filter(groups::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("グループが見つかりません".into()))?;

        // ユーザーがグループのメンバーかどうか確認
        let is_member = group_members::Entity::find()
            .filter(group_members::Column::GroupId.eq(group_id))
            .filter(group_members::Column::UserId.eq(user_id))
            .filter(group_members::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .is_some();

        if !is_member {
            return Err(DbErr::Custom(
                "ユーザーはグループのメンバーではありません".into(),
            ));
        }

        // メッセージを取得
        let query = group_messages::Entity::find()
            .filter(group_messages::Column::GroupId.eq(group_id))
            .filter(group_messages::Column::DeletedAt.is_null())
            .order_by_desc(group_messages::Column::CreatedAt);

        let total_count = query.clone().count(&self.db).await?;
        
        // ページネーションの計算（0ベースのページ番号に変換）
        let page_num = if page <= 0 { 0 } else { (page - 1) as u64 };
        let messages = query
            .paginate(&self.db, per_page as u64)
            .fetch_page(page_num)
            .await?;

        // 未読メッセージ数を取得
        // 特定のグループの全メッセージを取得
        let all_messages = group_messages::Entity::find()
            .filter(group_messages::Column::GroupId.eq(group_id))
            .filter(group_messages::Column::DeletedAt.is_null())
            .all(&self.db)
            .await?;

        // 既読メッセージのIDを取得
        let read_message_ids = group_message_reads::Entity::find()
            .filter(group_message_reads::Column::GroupId.eq(group_id))
            .filter(group_message_reads::Column::UserId.eq(user_id))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|r| r.message_id)
            .collect::<Vec<i32>>();

        // 未読メッセージをカウント
        let unread_count = all_messages
            .iter()
            .filter(|m| !read_message_ids.contains(&m.id))
            .count() as i32;

        // 各メッセージの既読ユーザーIDを取得
        let mut messages_with_read_info = Vec::new();
        for message in messages {
            let read_by_users = group_message_reads::Entity::find()
                .filter(group_message_reads::Column::MessageId.eq(message.id))
                .all(&self.db)
                .await?
                .into_iter()
                .map(|r| r.user_id)
                .collect::<Vec<i32>>();

            messages_with_read_info.push((message, read_by_users));
        }

        Ok((
            messages_with_read_info,
            total_count as i32,
            unread_count as i32,
        ))
    }

    async fn mark_group_message_as_read(
        &self,
        group_id: i32,
        message_id: i32,
        user_id: i32,
    ) -> Result<bool, DbErr> {
        // メッセージが存在するか確認
        let _message = group_messages::Entity::find_by_id(message_id)
            .filter(group_messages::Column::GroupId.eq(group_id))
            .filter(group_messages::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("メッセージが見つかりません".into()))?;

        // ユーザーがグループのメンバーかどうか確認
        let is_member = group_members::Entity::find()
            .filter(group_members::Column::GroupId.eq(group_id))
            .filter(group_members::Column::UserId.eq(user_id))
            .filter(group_members::Column::DeletedAt.is_null())
            .one(&self.db)
            .await?
            .is_some();

        if !is_member {
            return Err(DbErr::Custom(
                "ユーザーはグループのメンバーではありません".into(),
            ));
        }

        // 既に既読かどうか確認
        let existing_read = group_message_reads::Entity::find()
            .filter(group_message_reads::Column::GroupId.eq(group_id))
            .filter(group_message_reads::Column::MessageId.eq(message_id))
            .filter(group_message_reads::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?;

        if existing_read.is_some() {
            return Ok(true); // 既に既読
        }

        // 既読レコードを作成
        let now = Utc::now().naive_utc();
        let new_read = group_message_reads::ActiveModel {
            id: NotSet,
            group_id: Set(group_id),
            message_id: Set(message_id),
            user_id: Set(user_id),
            read_at: Set(now),
        };

        let res = group_message_reads::Entity::insert(new_read)
            .exec(&self.db)
            .await?;

        Ok(res.last_insert_id > 0)
    }

    async fn delete_group_message(&self, message_id: i32) -> Result<bool, DbErr> {
        let now = Utc::now().naive_utc();
        let result = group_messages::Entity::update_many()
            .filter(group_messages::Column::Id.eq(message_id))
            .filter(group_messages::Column::DeletedAt.is_null())
            .col_expr(group_messages::Column::DeletedAt, Expr::value(Some(now)))
            .col_expr(group_messages::Column::UpdatedAt, Expr::value(now))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::groups::Model as Group;
    use crate::domain::repository::group::GroupRepository;
    use dotenv::dotenv;
    use sea_orm::{Database, DatabaseConnection};
    use std::env;
    use tokio;

    // テスト用データベース接続をセットアップする関数
    async fn setup_test_db() -> DatabaseConnection {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Database::connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    // ダミーユーザを users テーブルへ挿入する
    async fn insert_dummy_user(db: &DatabaseConnection) -> i32 {
        use crate::domain::entity::users::{
            ActiveModel as UserActiveModel, Model as UserModel,
        };
        let dummy_user = UserActiveModel {
            id: NotSet,
            name: Set("dummy user".to_string()),
            email: Set("dummy@example.com".to_string()),
            description: NotSet,
            age: NotSet,
            gender: NotSet,
            address: NotSet,
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            deleted_at: NotSet,
        };
        let inserted: UserModel = dummy_user
            .insert(db)
            .await
            .expect("Insert dummy user failed");
        inserted.id
    }

    // テスト用グループを作成して返す
    async fn create_test_group(
        repo: &PgGroupRepository,
        creator_id: i32,
        member_ids: Vec<i32>,
    ) -> Group {
        repo.create_group(
            "Test Group".to_string(),
            Some("Test Description".to_string()),
            creator_id,
            member_ids,
        )
        .await
        .expect("Create group failed")
    }

    #[tokio::test]
    async fn test_create_and_get_group() {
        let db = setup_test_db().await;
        let creator_id = insert_dummy_user(&db).await;
        let member_id = insert_dummy_user(&db).await;
        let repo = PgGroupRepository::new(db);

        // グループを作成
        let group = repo
            .create_group(
                "Test Group".to_string(),
                Some("Test Description".to_string()),
                creator_id,
                vec![member_id],
            )
            .await
            .expect("Create group failed");

        // 作成されたグループを確認
        assert!(group.id > 0);
        assert_eq!(group.name, "Test Group");
        assert_eq!(group.description, Some("Test Description".to_string()));
        assert_eq!(group.creator_id, creator_id);

        // get_group で取得
        let retrieved = repo.get_group(group.id).await.expect("Get group failed");
        assert!(retrieved.is_some());
        let retrieved_group = retrieved.unwrap();
        assert_eq!(retrieved_group.id, group.id);
        assert_eq!(retrieved_group.name, group.name);
        assert_eq!(retrieved_group.description, group.description);
    }

    #[tokio::test]
    async fn test_update_group() {
        let db = setup_test_db().await;
        let creator_id = insert_dummy_user(&db).await;
        let repo = PgGroupRepository::new(db);

        // グループを作成
        let group = create_test_group(&repo, creator_id, vec![]).await;

        // グループを更新
        let updated_name = "Updated Group Name".to_string();
        let updated_description = "Updated Description".to_string();
        let updated_group = repo
            .update_group(
                group.id,
                Some(updated_name.clone()),
                Some(updated_description.clone()),
            )
            .await
            .expect("Update group failed");

        // 更新されたグループを確認
        assert_eq!(updated_group.id, group.id);
        assert_eq!(updated_group.name, updated_name);
        assert_eq!(updated_group.description, Some(updated_description));
    }

    #[tokio::test]
    async fn test_delete_group() {
        let db = setup_test_db().await;
        let creator_id = insert_dummy_user(&db).await;
        let repo = PgGroupRepository::new(db);

        // グループを作成
        let group = create_test_group(&repo, creator_id, vec![]).await;

        // グループを削除
        let result = repo
            .delete_group(group.id)
            .await
            .expect("Delete group failed");
        assert!(result, "Delete operation should return true for success");

        // 削除後、get_group で取得、None となることを確認
        let retrieved = repo.get_group(group.id).await.expect("Get group failed");
        assert!(retrieved.is_none(), "Deleted group still exists");
    }

    #[tokio::test]
    async fn test_list_groups() {
        let db = setup_test_db().await;
        let user_id = insert_dummy_user(&db).await;
        let repo = PgGroupRepository::new(db);

        // 複数のグループを作成
        let group1 = create_test_group(&repo, user_id, vec![]).await;
        let group2 = create_test_group(&repo, user_id, vec![]).await;

        // ユーザーが所属するグループを取得
        let (groups, total_count) = repo
            .list_groups(user_id, 1, 10)
            .await
            .expect("List groups failed");

        // 結果を確認
        assert!(total_count >= 2, "Should have at least 2 groups");
        assert!(groups.len() >= 2, "Should return at least 2 groups");

        // グループIDの存在を確認
        let group_ids: Vec<i32> = groups.iter().map(|g| g.id).collect();
        assert!(
            group_ids.contains(&group1.id),
            "Group 1 not found in results"
        );
        assert!(
            group_ids.contains(&group2.id),
            "Group 2 not found in results"
        );
    }

    #[tokio::test]
    async fn test_add_and_remove_group_member() {
        let db = setup_test_db().await;
        let creator_id = insert_dummy_user(&db).await;
        let member_id = insert_dummy_user(&db).await;
        let repo = PgGroupRepository::new(db);

        // グループを作成
        let group = create_test_group(&repo, creator_id, vec![]).await;

        // メンバーを追加
        let member = repo
            .add_group_member(group.id, member_id, "member".to_string())
            .await
            .expect("Add member failed");

        // 追加したメンバーを確認
        assert_eq!(member.group_id, group.id);
        assert_eq!(member.user_id, member_id);
        assert_eq!(member.role, "member");

        // グループメンバー一覧を取得
        let (members, count) = repo
            .list_group_members(group.id, 1, 10)
            .await
            .expect("List members failed");

        // 結果を確認（作成者と追加したメンバーの2人）
        assert_eq!(count, 2, "Should have 2 members");
        assert_eq!(members.len(), 2, "Should return 2 members");

        // メンバーを削除
        let result = repo
            .remove_group_member(group.id, member_id)
            .await
            .expect("Remove member failed");
        assert!(result, "Remove operation should return true for success");

        // 削除後、メンバー一覧を再取得
        let (members_after, count_after) = repo
            .list_group_members(group.id, 1, 10)
            .await
            .expect("List members failed");

        // 結果を確認（作成者のみ）
        assert_eq!(count_after, 1, "Should have 1 member after removal");
        assert_eq!(
            members_after.len(),
            1,
            "Should return 1 member after removal"
        );
        assert_eq!(
            members_after[0].0.user_id, creator_id,
            "Remaining member should be the creator"
        );
    }

    #[tokio::test]
    async fn test_send_and_read_group_message() {
        let db = setup_test_db().await;
        let creator_id = insert_dummy_user(&db).await;
        let member_id = insert_dummy_user(&db).await;
        let repo = PgGroupRepository::new(db);

        // グループを作成（メンバーとして member_id を追加）
        let group = create_test_group(&repo, creator_id, vec![member_id]).await;

        // メッセージを送信
        let message_content = "Test message".to_string();
        let message = repo
            .send_group_message(group.id, creator_id, message_content.clone())
            .await
            .expect("Send message failed");

        // 送信したメッセージを確認
        assert_eq!(message.group_id, group.id);
        assert_eq!(message.sender_id, creator_id);
        assert_eq!(message.content, message_content);

        // グループメッセージ一覧を取得（creator_id として）
        let (messages, total_count, unread_count) = repo
            .list_group_messages(group.id, creator_id, 1, 10)
            .await
            .expect("List messages failed");

        // 結果を確認
        assert_eq!(total_count, 1, "Should have 1 message");
        assert_eq!(messages.len(), 1, "Should return 1 message");
        assert_eq!(messages[0].0.id, message.id, "Message ID should match");
        assert_eq!(
            unread_count, 0,
            "Creator should have 0 unread messages (auto-read)"
        );

        // メンバーがメッセージを読む
        let read_result = repo
            .mark_group_message_as_read(group.id, message.id, member_id)
            .await
            .expect("Mark message as read failed");
        assert!(
            read_result,
            "Mark as read operation should return true for success"
        );

        // メッセージを削除
        let delete_result = repo
            .delete_group_message(message.id)
            .await
            .expect("Delete message failed");
        assert!(
            delete_result,
            "Delete operation should return true for success"
        );

        // 削除後、メッセージ一覧を再取得
        let (messages_after, total_count_after, _) = repo
            .list_group_messages(group.id, creator_id, 1, 10)
            .await
            .expect("List messages failed");

        // 結果を確認
        assert_eq!(
            total_count_after, 0,
            "Should have 0 messages after deletion"
        );
        assert_eq!(
            messages_after.len(),
            0,
            "Should return 0 messages after deletion"
        );
    }
}
