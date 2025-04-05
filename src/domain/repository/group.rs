use crate::domain::entity::{group_members, group_messages, groups};
use sea_orm::DbErr;

#[async_trait::async_trait]
pub trait GroupRepository {
    /// グループを作成します。
    /// - `name`: グループ名
    /// - `description`: グループの説明
    /// - `creator_id`: 作成者のユーザーID
    /// - `initial_member_ids`: 初期メンバーのユーザーID一覧
    ///
    /// 成功時は作成されたグループ（Entity）を返します。
    async fn create_group(
        &self,
        name: String,
        description: Option<String>,
        creator_id: i32,
        initial_member_ids: Vec<i32>,
    ) -> Result<groups::Model, DbErr>;

    /// グループ情報を取得します。
    /// - `group_id`: グループID
    ///
    /// 成功時はグループ（Entity）を返します。
    async fn get_group(&self, group_id: i32) -> Result<Option<groups::Model>, DbErr>;

    /// ユーザーが所属するグループ一覧を取得します。
    /// - `user_id`: ユーザーID
    /// - `page` と `per_page`: ページネーション用
    ///
    /// 返り値は、(グループリスト, 全件数) のタプルです。
    async fn list_groups(
        &self,
        user_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<groups::Model>, i32), DbErr>;

    /// グループ情報を更新します。
    /// - `group_id`: グループID
    /// - `name`: 新しいグループ名（Noneの場合は更新しない）
    /// - `description`: 新しいグループの説明（Noneの場合は更新しない）
    ///
    /// 成功時は更新されたグループ（Entity）を返します。
    async fn update_group(
        &self,
        group_id: i32,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<groups::Model, DbErr>;

    /// グループを削除します（論理削除）。
    /// - `group_id`: グループID
    ///
    /// 成功時は true を返します。
    async fn delete_group(&self, group_id: i32) -> Result<bool, DbErr>;

    /// グループにメンバーを追加します。
    /// - `group_id`: グループID
    /// - `user_id`: 追加するユーザーID
    /// - `role`: メンバーの役割（"admin", "member" など）
    ///
    /// 成功時は追加されたメンバー（Entity）を返します。
    async fn add_group_member(
        &self,
        group_id: i32,
        user_id: i32,
        role: String,
    ) -> Result<group_members::Model, DbErr>;

    /// グループからメンバーを削除します（論理削除）。
    /// - `group_id`: グループID
    /// - `user_id`: 削除するユーザーID
    ///
    /// 成功時は true を返します。
    async fn remove_group_member(&self, group_id: i32, user_id: i32) -> Result<bool, DbErr>;

    /// グループのメンバー一覧を取得します。
    /// - `group_id`: グループID
    /// - `page` と `per_page`: ページネーション用
    ///
    /// 返り値は、(メンバーリスト, 全件数) のタプルです。
    async fn list_group_members(
        &self,
        group_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<(group_members::Model, Option<crate::domain::entity::users::Model>)>, i32), DbErr>;

    /// グループにメッセージを送信します。
    /// - `group_id`: グループID
    /// - `sender_id`: 送信者のユーザーID
    /// - `content`: メッセージ本文
    ///
    /// 成功時は送信されたメッセージ（Entity）を返します。
    async fn send_group_message(
        &self,
        group_id: i32,
        sender_id: i32,
        content: String,
    ) -> Result<group_messages::Model, DbErr>;

    /// グループのメッセージ一覧を取得します。
    /// - `group_id`: グループID
    /// - `user_id`: リクエストを送信するユーザーID
    /// - `page` と `per_page`: ページネーション用
    ///
    /// 返り値は、(メッセージリスト, 全件数, 未読件数) のタプルです。
    /// メッセージリストには送信者情報と既読情報も含まれます。
    async fn list_group_messages(
        &self,
        group_id: i32,
        user_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<(group_messages::Model, Vec<i32>)>, i32, i32), DbErr>;

    /// グループメッセージを既読にします。
    /// - `group_id`: グループID
    /// - `message_id`: メッセージID
    /// - `user_id`: 既読にするユーザーID
    ///
    /// 成功時は true を返します。
    async fn mark_group_message_as_read(
        &self,
        group_id: i32,
        message_id: i32,
        user_id: i32,
    ) -> Result<bool, DbErr>;

    /// グループメッセージを削除します（論理削除）。
    /// - `message_id`: メッセージID
    ///
    /// 成功時は true を返します。
    async fn delete_group_message(&self, message_id: i32) -> Result<bool, DbErr>;
}
