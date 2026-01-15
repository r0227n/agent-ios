fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false) // Client only
        .compile_protos(&["proto/idb.proto"], &["proto/"])?;
    Ok(())
}
