use crate::helpers::client::{with_client, CommandResult};
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

pub async fn add_media(file_paths: Vec<String>, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        client.add_media(file_paths).await?;
        Ok(())
    })
    .await
}
