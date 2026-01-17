pub mod ls;
pub mod mkdir;
pub mod mv;
pub mod pull;
pub mod push;
pub mod rm;
pub mod tail;

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

    /// Pull a file from the target device
    Pull {
        /// Source path on device
        src_path: String,

        /// Destination path on host (use "-" for stdout)
        dst_path: String,

        /// Bundle ID for app-specific file container
        #[arg(long)]
        bundle_id: Option<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Push a file to the target device
    Push {
        /// Source path on host
        src_path: String,

        /// Destination path on device
        dst_path: String,

        /// Bundle ID for app-specific file container
        #[arg(long)]
        bundle_id: Option<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Tail a file on the target device
    Tail {
        /// Path to file to tail
        path: String,

        /// Bundle ID for app-specific file container
        #[arg(long)]
        bundle_id: Option<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}
