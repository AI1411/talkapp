use crate::post_proto::post_service_server::PostService;
use crate::post_proto::{
    CreatePostRequest, CreatePostResponse, DeletePostRequest, DeletePostResponse, GetPostRequest,
    GetPostResponse, ListPostsRequest, ListPostsResponse, Post,
};
use crate::usecase::post_usecase::PostUseCase;
use sea_orm::DbErr;
use tonic::{Request, Response, Status};

pub struct PostHandler<U> {
    usecase: U,
}

impl<U: PostUseCase> PostHandler<U> {
    pub fn new(usecase: U) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl<U: PostUseCase + Send + Sync + 'static> PostService for PostHandler<U> {
    async fn create_post(
        &self,
        request: Request<CreatePostRequest>,
    ) -> Result<Response<CreatePostResponse>, Status> {
        let req = request.into_inner();
        let post = self
            .usecase
            .create_post(req.body, req.user_id as i32)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreatePostResponse {
            post: Some(Post {
                id: post.id as u64,
                body: post.body,
                user_id: post.user_id as u64,
                created_at: post.created_at,
            }),
        }))
    }

    async fn list_posts(
        &self,
        request: Request<ListPostsRequest>,
    ) -> Result<Response<ListPostsResponse>, Status> {
        let _req = request.into_inner();
        let posts = self
            .usecase
            .list_posts()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let posts = posts
            .into_iter()
            .map(|p| Post {
                id: p.id as u64,
                body: p.body,
                user_id: p.user_id as u64,
                created_at: p.created_at,
            })
            .collect();

        Ok(Response::new(ListPostsResponse { posts }))
    }

    async fn get_post(
        &self,
        request: Request<GetPostRequest>,
    ) -> Result<Response<GetPostResponse>, Status> {
        let req = request.into_inner();
        let post = self
            .usecase
            .get_post(req.id as i32)
            .await
            .map_err(|e| match e {
                DbErr::RecordNotFound(_) => Status::not_found("Post not found"),
                _ => Status::internal(e.to_string()),
            })?
            .ok_or_else(|| Status::not_found("Post not found"))?;

        Ok(Response::new(GetPostResponse {
            post: Some(Post {
                id: post.id as u64,
                body: post.body,
                user_id: post.user_id as u64,
                created_at: post.created_at,
            }),
        }))
    }

    async fn delete_post(
        &self,
        request: Request<DeletePostRequest>,
    ) -> Result<Response<DeletePostResponse>, Status> {
        let req = request.into_inner();

        self.usecase
            .delete_post(req.id as i32)
            .await
            .map_err(|e| match e {
                DbErr::RecordNotFound(_) => Status::not_found("Post not found"),
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(DeletePostResponse { success: true }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::post;
    use crate::usecase::post_usecase::PostUseCase;
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;

    // PostUseCaseのモック
    mock! {
        pub PostUseCase {}

        #[async_trait]
        impl PostUseCase for PostUseCase {
            async fn create_post(&self, body: String, user_id: i32) -> Result<post::Model, DbErr>;
            async fn list_posts(&self) -> Result<Vec<post::Model>, DbErr>;
            async fn get_post(&self, id: i32) -> Result<Option<post::Model>, DbErr>;
            async fn delete_post(&self, id: i32) -> Result<(), DbErr>;
        }
    }

    fn create_test_post() -> post::Model {
        post::Model {
            id: 1,
            body: "テスト投稿".to_string(),
            user_id: 1,
            created_at: "2023-01-01 12:00:00".to_string(),
        }
    }

    #[tokio::test]
    async fn test_create_post() {
        let mut mock_usecase = MockPostUseCase::new();
        let test_post = create_test_post();
        
        mock_usecase
            .expect_create_post()
            .with(eq("テスト投稿".to_string()), eq(1))
            .returning(|_, _| Ok(create_test_post()));
        
        let handler = PostHandler::new(mock_usecase);
        let request = Request::new(CreatePostRequest {
            body: "テスト投稿".to_string(),
            user_id: 1,
        });
        
        let response = handler.create_post(request).await.unwrap();
        let post = response.into_inner().post.unwrap();
        
        assert_eq!(post.id, test_post.id as u64);
        assert_eq!(post.body, test_post.body);
        assert_eq!(post.user_id, test_post.user_id as u64);
    }

    #[tokio::test]
    async fn test_list_posts() {
        let mut mock_usecase = MockPostUseCase::new();
        let test_posts = vec![create_test_post()];
        
        mock_usecase
            .expect_list_posts()
            .returning(|| Ok(vec![create_test_post()]));
        
        let handler = PostHandler::new(mock_usecase);
        let request = Request::new(ListPostsRequest {
            page: 1,
            per_page: 10,
        });
        
        let response = handler.list_posts(request).await.unwrap();
        let posts = response.into_inner().posts;
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, test_posts[0].id as u64);
        assert_eq!(posts[0].body, test_posts[0].body);
    }

    #[tokio::test]
    async fn test_get_post() {
        let mut mock_usecase = MockPostUseCase::new();
        let test_post = create_test_post();
        
        mock_usecase
            .expect_get_post()
            .with(eq(1))
            .returning(|_| Ok(Some(create_test_post())));
        
        let handler = PostHandler::new(mock_usecase);
        let request = Request::new(GetPostRequest { id: 1 });
        
        let response = handler.get_post(request).await.unwrap();
        let post = response.into_inner().post.unwrap();
        
        assert_eq!(post.id, test_post.id as u64);
        assert_eq!(post.body, test_post.body);
    }

    #[tokio::test]
    async fn test_get_post_not_found() {
        let mut mock_usecase = MockPostUseCase::new();
        
        mock_usecase
            .expect_get_post()
            .with(eq(999))
            .returning(|_| Ok(None));
        
        let handler = PostHandler::new(mock_usecase);
        let request = Request::new(GetPostRequest { id: 999 });
        
        let result = handler.get_post(request).await;
        assert!(result.is_err());
        
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn test_delete_post() {
        let mut mock_usecase = MockPostUseCase::new();
        
        mock_usecase
            .expect_delete_post()
            .with(eq(1))
            .returning(|_| Ok(()));
        
        let handler = PostHandler::new(mock_usecase);
        let request = Request::new(DeletePostRequest { id: 1 });
        
        let response = handler.delete_post(request).await.unwrap();
        assert!(response.into_inner().success);
    }

    #[tokio::test]
    async fn test_delete_post_not_found() {
        let mut mock_usecase = MockPostUseCase::new();
        
        mock_usecase
            .expect_delete_post()
            .with(eq(999))
            .returning(|_| Err(DbErr::RecordNotFound("Post not found".to_string())));
        
        let handler = PostHandler::new(mock_usecase);
        let request = Request::new(DeletePostRequest { id: 999 });
        
        let result = handler.delete_post(request).await;
        assert!(result.is_err());
        
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }
}
