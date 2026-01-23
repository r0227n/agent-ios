use agent_mobile_platform_ios::proto::idb::video_stream_request::Format;

use crate::helpers::{setup_ctrl_c_handler, with_client, CommandResult};

/// Stream raw H264 from the target
pub async fn run(
    output_file: Option<String>,
    fps: Option<u64>,
    format: String,
    compression_quality: f64,
    scale_factor: f64,
    udid: Option<String>,
) -> CommandResult {
    let video_format = match format.to_lowercase().as_str() {
        "h264" => Format::H264,
        "rgba" => Format::Rgba,
        "mjpeg" => Format::Mjpeg,
        "minicap" => Format::Minicap,
        "i420" => Format::I420,
        _ => {
            return Err(format!(
                "Invalid video format '{}'. Valid options: h264, rgba, mjpeg, minicap, i420",
                format
            )
            .into())
        }
    };

    if let Some(ref path) = output_file {
        eprintln!("Streaming video to {} until ^C", path);
    } else {
        eprintln!("Streaming video to stdout until ^C");
    }

    let stop_rx = setup_ctrl_c_handler();

    with_client(udid.as_deref(), |mut client| async move {
        client
            .video_stream(
                output_file,
                fps,
                video_format,
                compression_quality,
                scale_factor,
                stop_rx,
            )
            .await?;
        eprintln!("\nStreaming stopped");
        Ok(())
    })
    .await
}
