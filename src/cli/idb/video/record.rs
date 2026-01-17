use crate::cli::helpers::{setup_ctrl_c_handler, with_client, CommandResult};

/// Record the target's screen to a mp4 video file
pub async fn run(output_file: String, udid: Option<String>) -> CommandResult {
    eprintln!("Recording video to {} until ^C", output_file);

    let stop_rx = setup_ctrl_c_handler();

    with_client(udid.as_deref(), |mut client| async move {
        client.record_video(output_file, stop_rx).await?;
        eprintln!("\nRecording stopped");
        Ok(())
    })
    .await
}
