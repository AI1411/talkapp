pub mod domain;
pub mod handler;
pub mod repository;
pub mod usecase;

// プロトコルバッファの生成ファイル
pub mod user_proto {
    tonic::include_proto!("user");
}
pub mod post_proto {
    tonic::include_proto!("post");
}
pub mod message_proto {
    tonic::include_proto!("message");
}
pub mod reaction_proto {
    tonic::include_proto!("reaction");
}
pub mod group_proto {
    tonic::include_proto!("group");
}