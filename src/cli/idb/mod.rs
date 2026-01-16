pub mod focus;
pub mod install;
pub mod kill;
pub mod launch;
pub mod list_targets;
pub mod log;
pub mod rm;
pub mod screenshot;
pub mod uninstall;
pub mod xctest_list;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum IdbCommands {
    /// Lists connected and available targets
    ListTargets {
        /// Which device types to list (device, simulator, mac)
        #[arg(long)]
        only: Option<String>,

        /// Output in human-readable format (like idb)
        #[arg(long)]
        human: bool,
    },

    /// Launch an application
    Launch {
        /// Application bundle identifier
        bundle_id: String,

        /// Arguments to pass to the application
        #[arg(trailing_var_arg = true)]
        app_arguments: Vec<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,

        /// Wait for debugger to attach
        #[arg(short = 'd', long)]
        wait_for_debugger: bool,

        /// Bring app to foreground if already running
        #[arg(short = 'f', long)]
        foreground_if_running: bool,

        /// Wait for process to exit, tailing output
        #[arg(short = 'w', long)]
        wait_for: bool,

        /// Write process PID to this file
        #[arg(short = 'p', long)]
        pid_file: Option<String>,
    },

    /// Kill the idb daemon
    Kill,

    /// Take a screenshot of the target device
    Screenshot {
        /// Destination path for the screenshot or "-" for stdout
        dest_path: String,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Bring simulator window to front
    Focus {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Obtain logs from the target or companion
    Log {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,

        /// Log source: target (default) or companion
        #[arg(long, value_parser = ["target", "companion"], default_value = "target")]
        source: String,

        /// Additional log arguments (e.g., --style json --level info)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        log_arguments: Vec<String>,
    },

    /// Install an application
    Install {
        /// Path to .app or .ipa bundle
        bundle_path: String,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,

        /// Make app debuggable (persist app bundle for debugserver)
        #[arg(long)]
        make_debuggable: bool,

        /// Override modification time of files in .ipa
        #[arg(long)]
        override_mtime: bool,

        /// Compression type (GZIP, ZSTD)
        #[arg(long)]
        compression: Option<String>,

        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },

    /// Uninstall an application
    Uninstall {
        /// Application bundle identifier
        bundle_id: String,

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

    /// List installed XCTest bundles
    XctestList {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}
