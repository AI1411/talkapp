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
            .map_err(|_e| DbErr::RecordNotFound(format!("Post with id {} not found", id)))
    }

    async fn find_by_user_id(&self, user_id: i32) -> Result<Vec<Post>, DbErr> {
        Posts::find()
            .filter(Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|e| DbErr::Exec(RuntimeErr::Internal(format!("Error: {}", e.to_string()))))
    }

    async fn insert(&self, body: String, user_id: i32) -> Result<Post, DbErr> {
        let _now = chrono::Utc::now().naive_utc();
        let post_data = post::ActiveModel {
            id: NotSet,
            body: Set(body.clone()),
            user_id: Set(user_id),
            ..Default::default()
        };

        post_data
            .insert(&self.db)
            .await
            .map_err(|e| DbErr::Exec(RuntimeErr::Internal(format!("Error: {}", e.to_string()))))
    }

    async fn update(&self, id: i32, body: String) -> Result<Post, DbErr> {
        let existing_post = Posts::find_by_id(id)
            .one(&self.db)
            .await
            .map_err(|e| DbErr::Custom(format!("Error retrieving post: {}", e)))?
            .ok_or(DbErr::RecordNotFound(format!(
                "Post with id {} not found",
                id
            )))?;

        let mut active_post: post::ActiveModel = existing_post.into();
        active_post.body = Set(body.clone());

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

    // テスト用データベース接続をセットアップする関数
    async fn setup_test_db() -> DatabaseConnection {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Database::connect(&database_url)
            .await
            .expect("Failed to connect to database")
    }

    // 修正: ダミーユーザを実際に users テーブルへ挿入する
    async fn insert_dummy_user(db: &DatabaseConnection) -> i32 {
        use crate::domain::entity::users::{
            ActiveModel as UserActiveModel, Entity as Users, Model as UserModel,
        };
        let dummy_user = UserActiveModel {
            id: NotSet,
            name: Set("dummy user".to_string()),
            email: Set("dummy@example.com".to_string()),
            description: NotSet,
            age: NotSet,
            gender: NotSet,
            address: NotSet,
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
            deleted_at: NotSet,
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
        // 実際に dummy user を挿入して有効な user_id を取得
        let dummy_user_id = insert_dummy_user(&db).await;
        let repo = PgPostRepository::new(db);

        // insert を body と user_id で呼び出す
        let inserted_post = repo
            .insert("Test post body".to_string(), dummy_user_id)
            .await
            .expect("Insert failed");
        assert!(inserted_post.id > 0);

        // get_by_id で取得
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
        let inserted_post = repo
            .insert("Original body".to_string(), dummy_user_id)
            .await
            .expect("Insert failed");

        // update: id と新しい body を渡す
        let updated_post = repo
            .update(inserted_post.id, "Updated body".to_string())
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
        let inserted_post = repo
            .insert("Post to be deleted".to_string(), dummy_user_id)
            .await
            .expect("Insert failed");

        // delete を呼び出し
        repo.delete(inserted_post.id).await.expect("Delete failed");

        // 削除後、get_by_id で取得し、None となることを確認
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

        // 同一 user_id のレコードを2件挿入
        let _ = repo
            .insert("User post 1".to_string(), dummy_user_id)
            .await
            .expect("Insert post1 failed");
        let _ = repo
            .insert("User post 2".to_string(), dummy_user_id)
            .await
            .expect("Insert post2 failed");

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
