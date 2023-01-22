use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["proto/vector_tile.proto"], &["proto/"])?;
    Ok(())
}
