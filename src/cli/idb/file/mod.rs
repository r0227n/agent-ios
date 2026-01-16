pub mod ls;
pub mod rm;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum FileCommands {
    /// List files on the target device
    Ls {
        /// Paths to list
        #[arg(required = true)]
        paths: Vec<String>,

        /// Bundle ID for app-specific file container
        #[arg(long)]
        bundle_id: Option<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Remove files or directories inside a container
    Rm {
        /// Paths to remove (directories will be recursively deleted)
        #[arg(required = true)]
        paths: Vec<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,

        /// Application bundle identifier (uses APPLICATION container)
        #[arg(long)]
        bundle_id: Option<String>,
    },
}
