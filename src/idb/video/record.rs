use crate::helpers::{setup_ctrl_c_handler, with_client, CommandResult};

/// Record the target's screen to a mp4 video file
pub async fn run(
    output_file: String,
    format: String,
    fps: Option<u64>,
    udid: Option<String>,
) -> CommandResult {
    eprintln!("Recording video to {} until ^C", output_file);

    let stop_rx = setup_ctrl_c_handler();

    with_client(udid.as_deref(), |mut client| async move {
        client
            .record_video(output_file, format, fps, stop_rx)
            .await?;
        eprintln!("\nRecording stopped");
        Ok(())
    })
    .await
}
