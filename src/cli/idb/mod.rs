pub mod focus;
pub mod install;
pub mod kill;
pub mod launch;
pub mod list_apps;
pub mod list_targets;
pub mod location;
pub mod log;
pub mod notification;
pub mod permissions;
pub mod rm;
pub mod screenshot;
pub mod terminate;
pub mod uninstall;
pub mod url;
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

    /// Grant app permissions
    Approve {
        /// Application bundle identifier
        bundle_id: String,

        /// Permissions to grant (photos, camera, contacts, url, location, notification, microphone)
        #[arg(required = true)]
        permissions: Vec<String>,

        /// URL scheme for URL permissions
        #[arg(long)]
        scheme: Option<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// List installed applications
    ListApps {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Set the device location
    Location {
        /// Subcommand for location operations
        #[command(subcommand)]
        command: LocationCommands,
    },

    /// Send push notifications to the device
    Notification {
        /// Subcommand for notification operations
        #[command(subcommand)]
        command: NotificationCommands,
    },

    /// Revoke app permissions
    Revoke {
        /// Application bundle identifier
        bundle_id: String,

        /// Permissions to revoke (photos, camera, contacts, url, location, notification, microphone)
        #[arg(required = true)]
        permissions: Vec<String>,

        /// URL scheme for URL permissions
        #[arg(long)]
        scheme: Option<String>,

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

    /// Terminate a running application
    Terminate {
        /// Application bundle identifier
        bundle_id: String,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Open a URL on the target device
    Url {
        /// Subcommand for URL operations
        #[command(subcommand)]
        command: UrlCommands,
    },

    /// List installed XCTest bundles
    XctestList {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum LocationCommands {
    /// Set device location to specific coordinates
    SetLocation {
        /// Latitude coordinate
        latitude: f64,

        /// Longitude coordinate
        longitude: f64,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum UrlCommands {
    /// Open a URL on the device
    Open {
        /// URL to open
        url: String,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum NotificationCommands {
    /// Send a simulated push notification
    SendNotification {
        /// Application bundle identifier
        bundle_id: String,

        /// JSON payload for the notification
        json_payload: String,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}
