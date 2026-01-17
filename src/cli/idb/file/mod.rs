pub mod ls;
pub mod mkdir;
pub mod mv;
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

    /// Create a directory inside a container
    Mkdir {
        /// Path to create
        path: String,

        /// Application bundle identifier (uses APPLICATION container)
        #[arg(long)]
        bundle_id: Option<String>,

        /// Use root container
        #[arg(long)]
        root: bool,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Move files on the target device
    Mv {
        /// Source paths to move
        #[arg(required = true)]
        src_paths: Vec<String>,

        /// Destination path
        dst_path: String,

        /// Bundle ID for app-specific file container
        #[arg(long)]
        bundle_id: Option<String>,

        /// Use root file container
        #[arg(long)]
        root: bool,

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
