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
