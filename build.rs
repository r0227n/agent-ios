use std::path::Path;
use std::process::Command;

fn build_idb_companion() {
    let idb_companion_path = "idb/build/Build/Products/Release/idb_companion";

    // 既にビルド済みならスキップ
    if Path::new(idb_companion_path).exists() {
        println!("cargo:warning=idb_companion already exists, skipping build");
        return;
    }

    println!("cargo:warning=Building idb_companion...");

    let status = Command::new("./build.sh")
        .arg("build")
        .arg("idb_companion")
        .current_dir("idb")
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:warning=idb_companion built successfully");
        }
        Ok(s) => {
            panic!("idb_companion build failed with: {}", s);
        }
        Err(e) => {
            panic!("Failed to build idb_companion: {}", e);
        }
    }

    // idb ディレクトリの変更を監視
    println!("cargo:rerun-if-changed=idb/build.sh");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // idb_companion を自動ビルド
    build_idb_companion();

    tonic_build::configure()
        .build_server(false) // Client only
        .type_attribute(
            "CrashLogInfo",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile_protos(&["proto/idb.proto"], &["proto/"])?;
    Ok(())
}
