mod infra;
mod repository;
mod domain;

use tonic::{transport::Server, Request, Response, Status};

mod user_proto {
    tonic::include_proto!("user");
}

use user_proto::user_service_server::{UserService, UserServiceServer};
use user_proto::{
    CreateUserRequest, CreateUserResponse, ListUsersRequest, ListUsersResponse,
    GetUserRequest, GetUserResponse, UpdateUserRequest, UpdateUserResponse,
    DeleteUserRequest, DeleteUserResponse,
};

#[derive(Clone, Default)]
pub struct MyUserService {
}

#[tonic::async_trait]
impl UserService for MyUserService {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();
        let reply = CreateUserResponse {
            id: 1,
            username: req.username,
            email: req.email,
            gender: req.gender,
            address: req.address,
            self_introduction: req.self_introduction,
        };
        Ok(Response::new(reply))
    }

    async fn list_users(
        &self,
        _request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let response = ListUsersResponse { users: vec![] };
        Ok(Response::new(response))
    }

    async fn get_user(
        &self,
        _request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        Err(Status::unimplemented("get_user not implemented"))
    }

    async fn update_user(
        &self,
        _request: Request<UpdateUserRequest>,
    ) -> Result<Response<UpdateUserResponse>, Status> {
        Err(Status::unimplemented("update_user not implemented"))
    }

    async fn delete_user(
        &self,
        _request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        Err(Status::unimplemented("delete_user not implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let user_service = MyUserService::default();

    println!("UserService gRPC Server listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(user_service))
        .serve(addr)
        .await?;

    Ok(())
}