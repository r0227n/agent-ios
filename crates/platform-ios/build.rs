fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false) // Client only
        .type_attribute(
            "CrashLogInfo",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_protos(&["proto/idb.proto"], &["proto/"])?;
    Ok(())
}
