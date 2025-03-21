use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::compile_protos("proto/user.proto")?;
    tonic_build::compile_protos("proto/post.proto")?;
    tonic_build::compile_protos("proto/message.proto")?;

    Ok(())
}
