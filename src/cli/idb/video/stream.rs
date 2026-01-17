use crate::cli::helpers::{setup_ctrl_c_handler, with_client, CommandResult};
use crate::grpc::idb::video_stream_request::Format;

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
        "rbga" => Format::Rbga,
        "mjpeg" => Format::Mjpeg,
        "minicap" => Format::Minicap,
        "i420" => Format::I420,
        _ => {
            return Err(format!(
                "Invalid video format '{}'. Valid options: h264, rbga, mjpeg, minicap, i420",
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
