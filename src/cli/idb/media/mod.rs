pub mod add_media;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum MediaCommands {
    /// Add photos/videos to the device
    AddMedia {
        /// Paths to media files
        #[arg(required = true)]
        file_paths: Vec<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}
