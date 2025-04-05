use crate::domain::entity::messages;
use sea_orm::DbErr;

#[async_trait::async_trait]
pub trait MessageRepository {
    /// 新規メッセージを送信します。
    /// - `sender_id`: 送信者のユーザーID
    /// - `receiver_id`: 受信者のユーザーID
    /// - `content`: メッセージ本文
    ///
    /// 成功時は送信されたメッセージ（Entity）を返します。
    async fn send_message(
        &self,
        sender_id: i32,
        receiver_id: i32,
        content: String,
    ) -> Result<messages::Model, DbErr>;

    /// ユーザーのメッセージ一覧を取得します。
    /// - `user_id`: 対象ユーザーのID
    /// - `unread_only`: 未読のみ取得する場合は true
    /// - `page` と `per_page`: ページネーション用
    ///
    /// 返り値は、(メッセージリスト, 全件数, 未読件数) のタプルです。
    async fn list_messages(
        &self,
        user_id: i32,
        unread_only: bool,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<messages::Model>, i32, i32), DbErr>;

    /// ユーザー間の会話履歴を取得します。
    /// - `user_id`: リクエストを送信するユーザーのID
    /// - `peer_id`: 会話相手のユーザーID
    /// - `page` と `per_page`: ページネーション用
    ///
    /// 返り値は、(メッセージリスト, 全件数) のタプルです。
    async fn get_conversation(
        &self,
        user_id: i32,
        peer_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<messages::Model>, i32), DbErr>;

    /// 指定されたメッセージまたはユーザー間のメッセージを既読に更新します。
    /// - 単一の `message_id` を指定する場合や、
    /// - 複数の `message_ids`、あるいは特定ユーザー間の全メッセージ更新を行うために `from_user_id` / `to_user_id` を指定できます。
    ///
    /// 更新された件数を返します。
    async fn mark_as_read(
        &self,
        message_id: Option<i32>,
        message_ids: Vec<i32>,
        from_user_id: Option<i32>,
        to_user_id: Option<i32>,
    ) -> Result<i32, DbErr>;

    /// 指定されたメッセージを削除します（論理削除など）。
    /// 成功時は true を返します。
    async fn delete_message(&self, message_id: i32) -> Result<bool, DbErr>;
}
