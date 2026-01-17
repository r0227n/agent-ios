use crate::cli::helpers::{file_container, with_client, CommandResult};
use std::io::Write;

/// Read a file from the target device and output to stdout
pub async fn run(
    src_path: String,
    udid: Option<String>,
    bundle_id: Option<String>,
) -> CommandResult {
    let container = file_container(bundle_id);

    // Create a temporary directory for the download
    let temp_dir = tempfile::tempdir()?;
    let file_name = std::path::Path::new(&src_path)
        .file_name()
        .ok_or("Invalid source path")?;
    let temp_file_path = temp_dir.path().join(file_name);
    let dest_dir = temp_dir.path().to_string_lossy().to_string();

    with_client(udid.as_deref(), |mut client| async move {
        // Pull file to temporary directory (Python idb does this too)
        client.pull_to_file(src_path, dest_dir, container).await?;

        // Read the file and write to stdout
        let data = std::fs::read(&temp_file_path)?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(&data)?;
        handle.flush()?;

        Ok(())
    })
    .await
}
