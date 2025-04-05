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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::users;
    use crate::usecase::user_usecase::UserUseCase;
    use async_trait::async_trait;
    use mockall::predicate::*;
    use mockall::*;
    use sqlx::Error as SqlxError;

    // UserUseCaseのモック
    mock! {
        pub UserUseCase {}

        #[async_trait]
        impl UserUseCase for UserUseCase {
            async fn create_user(
                &self,
                name: String,
                email: String,
                description: Option<String>,
                age: Option<i32>,
                gender: Option<String>,
                address: Option<String>
            ) -> Result<users::Model, SqlxError>;

            async fn list_users(&self) -> Result<Vec<users::Model>, SqlxError>;
            async fn get_user(&self, id: i32) -> Result<users::Model, SqlxError>;

            async fn update_user(
                &self,
                id: i32,
                name: Option<String>,
                email: Option<String>,
                description: Option<String>,
                age: Option<i32>,
                gender: Option<String>,
                address: Option<String>,
            ) -> Result<users::Model, SqlxError>;

            async fn delete_user(&self, id: i32) -> Result<users::Model, SqlxError>;
        }
    }

    fn create_test_user() -> users::Model {
        let dt = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        users::Model {
            id: 1,
            name: "テストユーザー".to_string(),
            email: "test@example.com".to_string(),
            description: Some("テスト説明".to_string()),
            age: Some(30),
            gender: Some("男性".to_string()),
            address: Some("東京都".to_string()),
            created_at: dt,
            updated_at: dt,
            deleted_at: None,
        }
    }

    #[tokio::test]
    async fn test_create_user() {
        let mut mock_usecase = MockUserUseCase::new();
        let test_user = create_test_user();
        let test_user_clone = test_user.clone();

        mock_usecase
            .expect_create_user()
            .with(
                eq("テストユーザー".to_string()),
                eq("test@example.com".to_string()),
                eq(Some("テスト説明".to_string())),
                eq(Some(30)),
                eq(Some("男性".to_string())),
                eq(Some("東京都".to_string())),
            )
            .returning(move |_, _, _, _, _, _| Ok(test_user_clone.clone()));

        let handler = UserHandler::new(mock_usecase);
        let request = Request::new(CreateUserRequest {
            name: "テストユーザー".to_string(),
            email: "test@example.com".to_string(),
            description: Some("テスト説明".to_string()),
            age: 30,
            gender: Some("男性".to_string()),
            address: Some("東京都".to_string()),
        });

        let response = handler.create_user(request).await.unwrap();
        let user = response.into_inner().user.unwrap();

        assert_eq!(user.id, test_user.id as u64);
        assert_eq!(user.name.unwrap(), test_user.name);
        assert_eq!(user.email.unwrap(), test_user.email);
        assert_eq!(user.age, test_user.age.unwrap() as u32);
        assert_eq!(user.gender, test_user.gender);
        assert_eq!(user.address, test_user.address);
    }

    #[tokio::test]
    async fn test_list_users() {
        let mut mock_usecase = MockUserUseCase::new();
        let test_users = vec![create_test_user()];

        mock_usecase
            .expect_list_users()
            .returning(|| Ok(vec![create_test_user()]));

        let handler = UserHandler::new(mock_usecase);
        let request = Request::new(ListUsersRequest {
            page: 1,
            per_page: 10,
            gender: None,
            address: None,
            name: None,
        });
        let response = handler.list_users(request).await.unwrap();
        let users = response.into_inner().users;
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].id, test_users[0].id as u64);
        assert_eq!(users[0].name.as_ref().unwrap(), &test_users[0].name);
        assert_eq!(users[0].email.as_ref().unwrap(), &test_users[0].email);
    }

    #[tokio::test]
    async fn test_get_user() {
        let mut mock_usecase = MockUserUseCase::new();
        let test_user = create_test_user();

        mock_usecase
            .expect_get_user()
            .with(eq(1))
            .returning(|_| Ok(create_test_user()));

        let handler = UserHandler::new(mock_usecase);
        let request = Request::new(GetUserRequest { id: 1 });

        let response = handler.get_user(request).await.unwrap();
        let user = response.into_inner().user.unwrap();

        assert_eq!(user.id, test_user.id as u64);
        assert_eq!(user.name.unwrap(), test_user.name);
        assert_eq!(user.email.unwrap(), test_user.email);
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mut mock_usecase = MockUserUseCase::new();

        mock_usecase
            .expect_get_user()
            .with(eq(999))
            .returning(|_| Err(SqlxError::RowNotFound));

        let handler = UserHandler::new(mock_usecase);
        let request = Request::new(GetUserRequest { id: 999 });

        let result = handler.get_user(request).await;
        assert!(result.is_err());

        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }

    #[tokio::test]
    async fn test_update_user() {
        let mut mock_usecase = MockUserUseCase::new();

        mock_usecase
            .expect_update_user()
            .with(
                eq(1),
                eq(Some("更新ユーザー".to_string())),
                eq(Some("updated@example.com".to_string())),
                eq(Some("更新説明".to_string())),
                eq(Some(35)),
                eq(Some("女性".to_string())),
                eq(Some("大阪府".to_string())),
            )
            .returning(|id, name, email, desc, age, gender, addr| {
                let dt = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap();

                Ok(users::Model {
                    id,
                    name: name.unwrap_or_default(),
                    email: email.unwrap_or_default(),
                    description: desc,
                    age,
                    gender,
                    address: addr,
                    created_at: dt,
                    updated_at: dt,
                    deleted_at: None,
                })
            });

        let handler = UserHandler::new(mock_usecase);
        let request = Request::new(UpdateUserRequest {
            id: 1,
            name: Some("更新ユーザー".to_string()),
            email: Some("updated@example.com".to_string()),
            description: Some("更新説明".to_string()),
            age: Some(35),
            gender: Some("女性".to_string()),
            address: Some("大阪府".to_string()),
        });

        let response = handler.update_user(request).await.unwrap();
        let user = response.into_inner().user.unwrap();

        assert_eq!(user.id, 1);
        assert_eq!(user.name.unwrap(), "更新ユーザー");
        assert_eq!(user.email.unwrap(), "updated@example.com");
        assert_eq!(user.age, 35);
        assert_eq!(user.gender, Some("女性".to_string()));
        assert_eq!(user.address, Some("大阪府".to_string()));
    }

    #[tokio::test]
    async fn test_delete_user() {
        let mut mock_usecase = MockUserUseCase::new();

        mock_usecase
            .expect_delete_user()
            .with(eq(1))
            .returning(|_| {
                let dt = chrono::NaiveDate::from_ymd_opt(2023, 1, 1)
                    .unwrap()
                    .and_hms_opt(12, 0, 0)
                    .unwrap();

                Ok(users::Model {
                    id: 1,
                    name: "".to_string(),
                    email: "".to_string(),
                    description: None,
                    age: None,
                    gender: None,
                    address: None,
                    created_at: dt,
                    updated_at: dt,
                    deleted_at: None,
                })
            });

        let handler = UserHandler::new(mock_usecase);
        let request = Request::new(DeleteUserRequest { id: 1 });
        let response = handler.delete_user(request).await.unwrap();
        assert_eq!(response.into_inner().success, true);
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let mut mock_usecase = MockUserUseCase::new();

        mock_usecase
            .expect_delete_user()
            .with(eq(999))
            .returning(|_| Err(SqlxError::RowNotFound));

        let handler = UserHandler::new(mock_usecase);
        let request = Request::new(DeleteUserRequest { id: 999 });

        let result = handler.delete_user(request).await;
        assert!(result.is_err());

        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::NotFound);
    }
}
