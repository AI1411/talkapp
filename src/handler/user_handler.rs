use crate::usecase::user_usecase::UserUseCase;
use crate::user_proto::user_service_server::UserService;
use crate::user_proto::{
    CreateUserRequest, CreateUserResponse, DeleteUserRequest, DeleteUserResponse, GetUserRequest,
    GetUserResponse, ListUsersRequest, ListUsersResponse, UpdateUserRequest, UpdateUserResponse,
    User,
};
use tonic::{Request, Response, Status};

pub struct UserHandler<U> {
    usecase: U,
}

impl<U: UserUseCase> UserHandler<U> {
    pub fn new(usecase: U) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl<U: UserUseCase + Send + Sync + 'static> UserService for UserHandler<U> {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        let user = self
            .usecase
            .create_user(
                req.name,
                req.email,
                req.description,
                Some(req.age as i32),
                req.gender,
                req.address,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateUserResponse {
            user: Some(User {
                id: user.id as u64,
                name: Some(user.name),
                email: Some(user.email),
                age: user.age.unwrap_or(0) as u32,
                address: user.address,
                description: user.description,
                gender: user.gender,
            }),
        }))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let _req = request.into_inner();
        let users = self
            .usecase
            .list_users()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let users = users
            .into_iter()
            .map(|u| User {
                id: u.id as u64,
                name: Some(u.name),
                email: Some(u.email),
                gender: u.gender,
                address: u.address,
                age: u.age.unwrap_or(0) as u32,
                description: u.description,
            })
            .collect();

        Ok(Response::new(ListUsersResponse { users }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();
        let user = self
            .usecase
            .get_user(req.id as i32)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Status::not_found("User not found"),
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(GetUserResponse {
            user: Some(User {
                id: user.id as u64,
                name: Some(user.name),
                email: Some(user.email),
                gender: user.gender,
                address: user.address,
                age: user.age.unwrap_or(0) as u32,
                description: user.description,
            }),
        }))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();

        // UpdateUserRequest の各フィールドを usecase の update_user に渡す
        let user = self
            .usecase
            .update_user(
                req.id as i32,
                req.name,
                req.email,
                req.description,
                req.age.map(|a| a as i32),
                req.gender,
                req.address,
            )
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Status::not_found("User not found"),
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(UpdateUserResponse {
            user: Some(User {
                id: user.id as u64,
                name: Some(user.name),
                email: Some(user.email),
                gender: user.gender,
                address: user.address,
                age: user.age.unwrap_or(0) as u32,
                description: user.description,
            }),
        }))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();

        self.usecase
            .delete_user(req.id as i32)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Status::not_found("User not found"),
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(DeleteUserResponse { success: true }))
    }
}
