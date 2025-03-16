use sea_orm::DbErr;
use crate::domain::entity::{reactions, reaction_types};

#[async_trait::async_trait]
pub trait ReactionRepository {
    /// メッセージにリアクションを追加します。
    /// - `user_id`: リアクションを付けるユーザーのID
    /// - `message_id`: リアクション対象のメッセージID
    /// - `reaction_type_id`: リアクションの種類ID
    ///
    /// 成功時はリアクションエンティティを返します。
    /// 同じユーザーが同じメッセージに同じリアクションを既に付けている場合はエラーを返します。
    async fn add_reaction(
        &self,
        user_id: i32,
        message_id: i32,
        reaction_type_id: i32,
    ) -> Result<reactions::Model, DbErr>;

    /// メッセージからリアクションを削除します。
    /// - `user_id`: リアクションを削除するユーザーのID
    /// - `message_id`: リアクション対象のメッセージID
    /// - `reaction_type_id`: リアクションの種類ID（指定しない場合は全種類）
    ///
    /// 成功時は削除されたリアクションの数を返します。
    async fn remove_reaction(
        &self,
        user_id: i32,
        message_id: i32,
        reaction_type_id: Option<i32>,
    ) -> Result<i32, DbErr>;

    /// メッセージに付けられたリアクションを取得します。
    /// - `message_id`: リアクション対象のメッセージID
    ///
    /// 成功時はリアクションエンティティのリストを返します。
    async fn get_reactions_for_message(
        &self,
        message_id: i32,
    ) -> Result<Vec<reactions::Model>, DbErr>;

    /// メッセージに付けられたリアクションを集計します。
    /// - `message_id`: リアクション対象のメッセージID
    ///
    /// 成功時はリアクションの種類ごとの数を返します。
    async fn count_reactions_by_type(
        &self,
        message_id: i32,
    ) -> Result<Vec<(reaction_types::Model, i64)>, DbErr>;

    /// 利用可能なリアクションの種類を全て取得します。
    ///
    /// 成功時はリアクションタイプエンティティのリストを返します。
    async fn list_reaction_types(&self) -> Result<Vec<reaction_types::Model>, DbErr>;

    /// 指定されたIDのリアクションタイプを取得します。
    /// - `id`: リアクションタイプのID
    ///
    /// 成功時はリアクションタイプエンティティを返します。
    async fn get_reaction_type(&self, id: i32) -> Result<Option<reaction_types::Model>, DbErr>;
}
