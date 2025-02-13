use tonic::Request;
use user_proto::user_service_client::UserServiceClient;
use user_proto::{CreateUserRequest, CreateUserResponse};

pub mod user_proto {
    tonic::include_proto!("user"); // .proto の package 名と合わせる
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // gRPC サーバーへの接続（例: localhost のポート 50051）
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;

    // CreateUser リクエストの作成
    let request = Request::new(CreateUserRequest {
        username: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        // これらは google.protobuf.StringValue 型となるため、必要なら None か Some(value) を渡します
        gender: None,
        address: None,
        self_introduction: None,
    });

    // gRPC の CreateUser RPC を呼び出す
    let response: tonic::Response<CreateUserResponse> = client.create_user(request).await?;

    println!("RESPONSE: {:?}", response.into_inner());
    Ok(())
}