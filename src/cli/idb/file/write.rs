use crate::cli::helpers::{file_container, with_client, CommandResult};
use std::io::Read;

/// Read from stdin and write to a file on the target device
pub async fn run(
    dst_path: String,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> CommandResult {
    // Read all data from stdin
    let mut stdin_data = Vec::new();
    std::io::stdin().read_to_end(&mut stdin_data)?;

    // Create a temporary file with the data
    let temp_dir = tempfile::tempdir()?;
    let file_name = std::path::Path::new(&dst_path)
        .file_name()
        .ok_or("Invalid destination path")?;
    let temp_file_path = temp_dir.path().join(file_name);

    std::fs::write(&temp_file_path, &stdin_data)?;

    // Extract the destination directory from dst_path
    let dst_dir = std::path::Path::new(&dst_path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();

    let container = file_container(bundle_id);

    // Move temp_dir into the closure to ensure it lives until push completes
    with_client(udid.as_deref(), |mut client| async move {
        let _temp_dir_guard = temp_dir; // Keep temp_dir alive
        client
            .push(
                temp_file_path.to_string_lossy().to_string(),
                dst_dir,
                container,
            )
            .await?;
        Ok(())
    })
    .await
}
