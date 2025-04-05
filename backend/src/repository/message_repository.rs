use crate::domain::entity::messages;
use crate::domain::repository::message::MessageRepository;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::Condition;
use sea_orm::QueryOrder;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, NotSet, Set};

pub struct PgMessageRepository {
    db: DatabaseConnection,
}

impl PgMessageRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MessageRepository for PgMessageRepository {
    async fn send_message(
        &self,
        sender_id: i32,
        receiver_id: i32,
        content: String,
    ) -> Result<messages::Model, DbErr> {
        let now = Utc::now().naive_utc();
        let new_message = messages::ActiveModel {
            id: NotSet,
            sender_id: Set(sender_id),
            receiver_id: Set(receiver_id),
            content: Set(content),
            is_read: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: NotSet,
        };
        // 挿入して、生成されたIDからエンティティを取得
        let res = messages::Entity::insert(new_message).exec(&self.db).await?;
        let message = messages::Entity::find_by_id(res.last_insert_id)
            .one(&self.db)
            .await?
            .ok_or(DbErr::Custom("メッセージの挿入に失敗しました".into()))?;
        Ok(message)
    }

    async fn list_messages(
        &self,
        user_id: i32,
        unread_only: bool,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<messages::Model>, i32, i32), DbErr> {
        // 受信者が対象のユーザーのメッセージを取得
        // 論理削除されていないメッセージのみを対象に
        let mut query = messages::Entity::find()
            .filter(messages::Column::ReceiverId.eq(user_id))
            .filter(messages::Column::DeletedAt.is_null());

        if unread_only {
            query = query.filter(messages::Column::IsRead.eq(false));
        }
        let total_count = query.clone().count(&self.db).await?;
        let paginator = query.paginate(&self.db, per_page as u64);
        let msgs = paginator.fetch_page(page as u64).await?;

        // 未読数も取得（削除されていないメッセージのみ）
        let unread_count = messages::Entity::find()
            .filter(messages::Column::ReceiverId.eq(user_id))
            .filter(messages::Column::IsRead.eq(false))
            .filter(messages::Column::DeletedAt.is_null())
            .count(&self.db)
            .await?;
        Ok((msgs, total_count as i32, unread_count as i32))
    }

    async fn get_conversation(
        &self,
        user_id: i32,
        peer_id: i32,
        page: i32,
        per_page: i32,
    ) -> Result<(Vec<messages::Model>, i32), DbErr> {
        // ユーザー間の会話：どちらが送信者でもOK
        let condition = Condition::any()
            .add(
                Condition::all()
                    .add(messages::Column::SenderId.eq(user_id))
                    .add(messages::Column::ReceiverId.eq(peer_id)),
            )
            .add(
                Condition::all()
                    .add(messages::Column::SenderId.eq(peer_id))
                    .add(messages::Column::ReceiverId.eq(user_id)),
            );

        // 論理削除されていないメッセージのみを対象に
        let query = messages::Entity::find()
            .filter(condition)
            .filter(messages::Column::DeletedAt.is_null())
            .order_by_desc(messages::Column::CreatedAt); // QueryOrderをインポートしたので問題なく使用可能

        let total_count = query.clone().count(&self.db).await?;
        let paginator = query.paginate(&self.db, per_page as u64);
        let msgs = paginator.fetch_page(page as u64).await?;
        Ok((msgs, total_count as i32))
    }

    // 他のメソッドは変更なし
    async fn mark_as_read(
        &self,
        message_id: Option<i32>,
        message_ids: Vec<i32>,
        from_user_id: Option<i32>,
        to_user_id: Option<i32>,
    ) -> Result<i32, DbErr> {
        let result = if message_id.is_some() || !message_ids.is_empty() {
            // 指定されたID（単体 or 複数）に対して更新
            let mut condition = Condition::any();
            if let Some(mid) = message_id {
                condition = condition.add(messages::Column::Id.eq(mid));
            }
            if !message_ids.is_empty() {
                condition = condition.add(messages::Column::Id.is_in(message_ids));
            }
            messages::Entity::update_many()
                .filter(condition)
                .filter(messages::Column::DeletedAt.is_null()) // 論理削除されていないものだけ
                .col_expr(messages::Column::IsRead, Expr::value(true))
                .col_expr(
                    messages::Column::UpdatedAt,
                    Expr::value(Utc::now().naive_utc()),
                )
                .exec(&self.db)
                .await?
        } else if from_user_id.is_some() && to_user_id.is_some() {
            // 特定のユーザー間の全メッセージを更新
            let from = from_user_id.unwrap();
            let to = to_user_id.unwrap();
            messages::Entity::update_many()
                .filter(messages::Column::SenderId.eq(from))
                .filter(messages::Column::ReceiverId.eq(to))
                .filter(messages::Column::DeletedAt.is_null()) // 論理削除されていないものだけ
                .col_expr(messages::Column::IsRead, Expr::value(true))
                .col_expr(
                    messages::Column::UpdatedAt,
                    Expr::value(Utc::now().naive_utc()),
                )
                .exec(&self.db)
                .await?
        } else {
            return Err(DbErr::Custom(
                "mark_as_read のための有効なパラメータが提供されませんでした".into(),
            ));
        };
        Ok(result.rows_affected as i32)
    }

    async fn delete_message(&self, message_id: i32) -> Result<bool, DbErr> {
        // 論理削除：deleted_at に現在時刻をセット
        let now = Utc::now().naive_utc();
        let result = messages::Entity::update_many()
            .filter(messages::Column::Id.eq(message_id))
            .filter(messages::Column::DeletedAt.is_null()) // 既に削除されていないものだけ
            .col_expr(messages::Column::DeletedAt, Expr::value(Some(now)))
            .col_expr(messages::Column::UpdatedAt, Expr::value(now))
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected > 0)
    }
}
