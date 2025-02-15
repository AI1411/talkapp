use crate::domain::entity::post;
use crate::domain::entity::post::{Column, Entity as Posts, Model as Post};
use crate::domain::repository::post::PostRepository;
use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{ColumnTrait, DatabaseConnection, NotSet, Set};

pub struct PgPostRepository {
    db: DatabaseConnection,
}

impl PgPostRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PostRepository for PgPostRepository {
    async fn find_all(&self) -> Result<Vec<Post>, DbErr> {
        Posts::find()
            .all(&self.db)
            .await
            .map_err(|e| DbErr::Exec(RuntimeErr::Internal(format!("Error: {}", e.to_string()))))
    }

    async fn get_by_id(&self, id: i32) -> Result<Option<Post>, DbErr> {
        Posts::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DbErr::RecordNotFound(format!("Post with id {} not found", id)))
    }

    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Post>, DbErr> {
        Posts::find()
            .filter(Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| DbErr::Exec(RuntimeErr::Internal(format!("Error: {}", e.to_string()))))
    }

    async fn insert(&self, post: &Post) -> Result<Post, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let post_data = post::ActiveModel {
            id: NotSet,
            body: Set(post.body.clone()),
            user_id: Set(post.user_id.clone()),
            ..Default::default()
        };

        post_data
            .insert(&self.db)
            .await
            .map_err(|e| DbErr::Exec(RuntimeErr::Internal(format!("Error: {}", e.to_string()))))
    }

    async fn update(&self, post: &Post) -> Result<Post, DbErr> {
        let existing_post = Posts::find_by_id(post.id)
            .one(&self.db)
            .await
            .map_err(|e| DbErr::Custom(format!("Error retrieving post: {}", e)))?
            .ok_or(DbErr::RecordNotFound(format!(
                "Post with id {} not found",
                post.id
            )))?;

        let mut active_post: post::ActiveModel = existing_post.into();

        active_post.body = Set(post.body.clone());
        active_post.user_id = Set(post.user_id);

        active_post
            .update(&self.db)
            .await
            .map_err(|e| DbErr::Exec(RuntimeErr::Internal(format!("Error updating post: {}", e))))
    }

    async fn delete(&self, id: i32) -> Result<(), DbErr> {
        Posts::delete_by_id(id)
            .exec(&self.db)
            .await
            .map_err(|e| sqlx::Error::Protocol(e.to_string().into()))
            .expect("delete post");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::post::Model as Post;
    use crate::domain::repository::post::PostRepository;
    use dotenv::dotenv;
    use sea_orm::{Database, DatabaseConnection, NotSet, Set};
    use std::env;
    use tokio;

    async fn setup_test_db() -> DatabaseConnection {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Database::connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    async fn insert_dummy_user(db: &DatabaseConnection) -> i32 {
        use crate::domain::entity::users::{ActiveModel as UserActiveModel, Model as UserModel};
        let dummy_user = UserActiveModel {
            id: NotSet,
            name: Set("dummy user".to_string()),
            ..Default::default()
        };

        let inserted: UserModel = dummy_user
            .insert(db)
            .await
            .expect("Insert dummy user failed");
        inserted.id
    }

    #[tokio::test]
    async fn test_insert_and_get_by_id() {
        let db = setup_test_db().await;
        // ダミーユーザを挿入して有効な user_id を取得
        let dummy_user_id = insert_dummy_user(&db).await;
        let repo = PgPostRepository::new(db);

        // テスト用の Post を作成（id は自動採番前提）
        let test_post = Post {
            id: 0,
            body: "Test post body".to_string(),
            user_id: dummy_user_id, // 有効な user_id を使用
            created_at: chrono::Utc::now().naive_utc().to_string(),
        };

        // insert
        let inserted_post = repo
            .insert(&test_post)
            .await
            .expect("Insert failed");
        assert!(inserted_post.id > 0);

        // get_by_id
        let retrieved = repo
            .get_by_id(inserted_post.id)
            .await
            .expect("Get by id failed");
        assert!(retrieved.is_some());
        let retrieved_post = retrieved.unwrap();
        assert_eq!(retrieved_post.body, inserted_post.body);
        assert_eq!(retrieved_post.user_id, inserted_post.user_id);
    }

    #[tokio::test]
    async fn test_update_post() {
        let db = setup_test_db().await;
        let dummy_user_id = insert_dummy_user(&db).await;
        let repo = PgPostRepository::new(db);

        // 新規レコードを挿入
        let original_post = Post {
            id: 0,
            body: "Original body".to_string(),
            user_id: dummy_user_id,
            created_at: chrono::Utc::now().naive_utc().to_string(),
        };
        let inserted_post = repo
            .insert(&original_post)
            .await
            .expect("Insert failed");

        // 更新用のデータを作成（body を変更）
        let updated_post_data = Post {
            id: inserted_post.id,
            body: "Updated body".to_string(),
            user_id: inserted_post.user_id, // ユーザIDはそのまま
            created_at: inserted_post.created_at.clone(),
        };

        let updated_post = repo
            .update(&updated_post_data)
            .await
            .expect("Update failed");
        assert_eq!(updated_post.body, "Updated body");
    }

    #[tokio::test]
    async fn test_delete_post() {
        let db = setup_test_db().await;
        let dummy_user_id = insert_dummy_user(&db).await;
        let repo = PgPostRepository::new(db);

        // 削除対象のレコードを挿入
        let test_post = Post {
            id: 0,
            body: "Post to be deleted".to_string(),
            user_id: dummy_user_id,
            created_at: chrono::Utc::now().naive_utc().to_string(),
        };
        let inserted_post = repo
            .insert(&test_post)
            .await
            .expect("Insert failed");

        // delete (ここでは id を指定して削除)
        repo.delete(inserted_post.id)
            .await
            .expect("Delete failed");

        // 削除後、get_by_id で取得して None になることを確認
        let result = repo.get_by_id(inserted_post.id).await;
        match result {
            Ok(opt) => assert!(opt.is_none(), "Deleted post still exists"),
            Err(e) => eprintln!("Error on get_by_id: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_find_by_user_id() {
        let db = setup_test_db().await;
        let dummy_user_id = insert_dummy_user(&db).await;
        let repo = PgPostRepository::new(db);

        // 同一の user_id のレコードを2件以上挿入
        let post1 = Post {
            id: 0,
            body: "User post 1".to_string(),
            user_id: dummy_user_id,
            created_at: chrono::Utc::now().naive_utc().to_string(),
        };
        let post2 = Post {
            id: 0,
            body: "User post 2".to_string(),
            user_id: dummy_user_id,
            created_at: chrono::Utc::now().naive_utc().to_string(),
        };
        // 異なるユーザのレコードも挿入（別の dummy user を用いるか、既存の有効な user_id を利用）
        let other_user_id = dummy_user_id + 1; // ※ 既存のユーザIDがある前提の場合はこちらを利用
        let post3 = Post {
            id: 0,
            body: "Other user post".to_string(),
            user_id: other_user_id,
            created_at: chrono::Utc::now().naive_utc().to_string(),
        };

        // ダミーユーザが存在するかどうか、other_user_id に対しては適切なレコード挿入が必要です。
        // ここではシンプルな例として、post1, post2 のみでテストします。

        let _ = repo.insert(&post1).await.expect("Insert post1 failed");
        let _ = repo.insert(&post2).await.expect("Insert post2 failed");

        let posts = repo
            .find_by_user_id(dummy_user_id)
            .await
            .expect("Find by user_id failed");
        // dummy_user_id の投稿が 2 件以上あることを確認
        assert!(posts.len() >= 2);
        for post in posts {
            assert_eq!(post.user_id, dummy_user_id);
        }
    }
}
