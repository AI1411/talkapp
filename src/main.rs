mod domain;
mod handler;mod repository;
mod usecase;

use crate::handler::message_handler::MessageHandler;
use crate::handler::post_handler::PostHandler;
use crate::handler::reaction_handler::ReactionHandler;
use crate::handler::user_handler::UserHandler;
use crate::repository::message_repository::PgMessageRepository;
use crate::repository::post_repository::PgPostRepository;
use crate::repository::reaction_repository::PgReactionRepository;
use crate::repository::user_repository::PgUserRepository;
use crate::usecase::message_usecase::MessageUseCaseImpl;
use crate::usecase::post_usecase::PostUseCaseImpl;
use crate::usecase::reaction_usecase::ReactionUseCaseImpl;
use crate::usecase::user_usecase::UserUseCaseImpl;
use dotenv::dotenv;
use tonic::transport::Server;

mod message_proto {
    tonic::include_proto!("message");
}

mod post_proto {
    tonic::include_proto!("post");
}

mod reaction_proto {
    tonic::include_proto!("reaction");
}

mod user_proto {
    tonic::include_proto!("user");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // DATABASE_URL 環境変数から接続先を取得
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // SeaORM を使ってデータベース接続を確立
    let pool = sea_orm::Database::connect(&database_url).await?;

    // リポジトリ、ユースケース、ハンドラを順次初期化
    let user_repository = PgUserRepository::new(pool.clone());
    let user_usecase = UserUseCaseImpl::new(user_repository);
    let user_handler = UserHandler::new(user_usecase);

    let post_repository = PgPostRepository::new(pool.clone());
    let post_usecase = PostUseCaseImpl::new(post_repository);
    let post_handler = PostHandler::new(post_usecase);

    let message_repository = PgMessageRepository::new(pool.clone());
    let message_usecase = MessageUseCaseImpl::new(message_repository);
    let message_handler = MessageHandler::new(message_usecase);

    let reaction_repository = PgReactionRepository::new(pool.clone());
    let reaction_usecase = ReactionUseCaseImpl::new(reaction_repository);
    let reaction_handler = ReactionHandler::new(reaction_usecase);

    let addr = "[::1]:50051".parse()?;
    println!("Server listening on {}", addr);

    // Tonic サーバーにハンドラを登録して起動
    Server::builder()
        .add_service(user_proto::user_service_server::UserServiceServer::new(
            user_handler,
        ))
        .add_service(post_proto::post_service_server::PostServiceServer::new(
            post_handler,
        ))
        .add_service(
            message_proto::message_service_server::MessageServiceServer::new(message_handler),
        )
        .add_service(
            reaction_proto::reaction_service_server::ReactionServiceServer::new(reaction_handler),
        )
        .serve(addr)
        .await?;

    Ok(())
}
