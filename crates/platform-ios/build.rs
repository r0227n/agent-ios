fn main() -> Result<(), Box<dyn std::error::Error>> {
    // proto directory is at project root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let proto_dir = std::path::Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("proto");
    let proto_file = proto_dir.join("core.proto");

    tonic_build::configure()
        .build_server(false) // Client only
        .compile_protos(&[proto_file], &[proto_dir])?;
    Ok(())
}
