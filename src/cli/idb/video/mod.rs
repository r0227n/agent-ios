pub mod record;
pub mod stream;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum VideoCommands {
    /// Record the target's screen to a mp4 video file
    #[command(name = "record-video")]
    RecordVideo {
        /// Output mp4 file path
        output_file: String,

        /// Format of the video (h264, rbga, mjpeg, minicap, i420)
        #[arg(long, default_value = "h264")]
        format: String,

        /// Framerate of the recording
        #[arg(long)]
        fps: Option<u64>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Stream raw H264 from the target
    #[command(name = "video-stream")]
    VideoStream {
        /// Output file path (omit for stdout)
        #[arg(short, long)]
        output_file: Option<String>,

        /// Framerate of the stream (default: dynamic)
        #[arg(long)]
        fps: Option<u64>,

        /// Format of the stream (h264, rbga, mjpeg, minicap, i420)
        #[arg(long, default_value = "h264")]
        format: String,

        /// Compression quality (0.0 to 1.0)
        #[arg(long, default_value = "0.2")]
        compression_quality: f64,

        /// Scale factor for the source video (0.0 to 1.0)
        #[arg(long, default_value = "1.0")]
        scale_factor: f64,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}
