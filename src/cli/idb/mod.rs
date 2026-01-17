pub mod accessibility;
pub mod contacts;
pub mod crash;
pub mod dap;
pub mod debugserver;
pub mod file;
pub mod focus;
pub mod hid;
pub mod install;
pub mod keychain;
pub mod kill;
pub mod launch;
pub mod list_apps;
pub mod list_targets;
pub mod location;
pub mod log;
pub mod media;
pub mod memory;
pub mod notification;
pub mod permissions;
pub mod photos;
pub mod screenshot;
pub mod settings;
pub mod target;
pub mod terminate;
pub mod uninstall;
pub mod url;
pub mod video;
pub mod xctest_install;
pub mod xctest_list;
pub mod xctest_list_bundle;
pub mod xctest_run;

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

    /// Crash log operations
    Crash {
        /// Subcommand for crash log operations
        #[command(subcommand)]
        command: CrashCommands,
    },

    /// File operations
    File {
        /// File operation subcommand
        #[command(subcommand)]
        command: file::FileCommands,
    },

    /// Tap on the screen
    Tap {
        /// X coordinate
        x: f64,
        /// Y coordinate
        y: f64,
        /// Press duration in seconds
        #[arg(long)]
        duration: Option<f64>,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Press a button
    Button {
        /// Button type (APPLE_PAY, HOME, LOCK, SIDE_BUTTON, SIRI)
        button: String,
        /// Press duration in seconds
        #[arg(long)]
        duration: Option<f64>,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Press a key
    Key {
        /// Key code
        keycode: u64,
        /// Press duration in seconds
        #[arg(long)]
        duration: Option<f64>,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Press a sequence of keys
    #[command(name = "key-sequence")]
    KeySequence {
        /// Key codes (space-separated)
        key_sequence: Vec<u64>,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Input text
    Text {
        /// Text to input
        text: String,
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Swipe from one point to another
    Swipe {
        /// X coordinate of swipe start
        x_start: f64,
        /// Y coordinate of swipe start
        y_start: f64,
        /// X coordinate of swipe end
        x_end: f64,
        /// Y coordinate of swipe end
        y_end: f64,
        /// Swipe duration in seconds
        #[arg(long)]
        duration: Option<f64>,
        /// Delta in pixels between touch points
        #[arg(long)]
        delta: Option<f64>,
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

    /// Device settings operations
    Settings {
        #[command(subcommand)]
        command: SettingsCommands,
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

    /// Media operations (photos/videos)
    Media {
        #[command(subcommand)]
        command: media::MediaCommands,
    },

    /// Video recording and streaming operations
    Video {
        #[command(subcommand)]
        command: video::VideoCommands,
    },

    /// Clear all photos from the device
    PhotosClear {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Get accessibility information for all elements
    AccessibilityDescribeAll {
        /// Use nested format (more detailed)
        #[arg(long)]
        nested: bool,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Get accessibility information at a specific point
    AccessibilityDescribePoint {
        /// X coordinate
        x: f64,

        /// Y coordinate
        y: f64,

        /// Use nested format (more detailed)
        #[arg(long)]
        nested: bool,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Update contacts database
    ContactsUpdate {
        /// Path to contacts database file
        db_path: String,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Clear all contacts
    ContactsClear {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Clear keychain
    KeychainClear {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Simulate memory warning
    MemoryWarning {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Install an XCTest bundle
    XctestInstall {
        /// Path to the XCTest bundle
        test_bundle_path: String,

        /// Skip signing the test bundle
        #[arg(long)]
        skip_signing: bool,

        /// Compression format (gzip or zstd)
        #[arg(long)]
        compression: Option<String>,

        /// Output in JSON format
        #[arg(long)]
        json: bool,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// List installed XCTest bundles
    XctestList {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// List tests inside an installed test bundle
    XctestListBundle {
        /// Bundle ID of the test bundle to list
        bundle_id: String,

        /// Path of the app of the test (needed for app tests)
        #[arg(long)]
        app_path: Option<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Run XCTest tests
    XctestRun {
        /// Test bundle ID to run
        test_bundle_id: String,

        /// Specific tests to run (format: ClassName/testMethod)
        #[arg(long)]
        tests_to_run: Vec<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Target/simulator management operations
    Target {
        #[command(subcommand)]
        command: target::TargetCommands,
    },

    /// Debug server operations
    Debugserver {
        #[command(subcommand)]
        command: debugserver::DebugServerCommands,
    },

    /// Spawn a debug server using VSCode DAP protocol
    Dap {
        /// Path of the DAP package to install
        dap_pkg_path: String,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum CrashCommands {
    /// List crash logs
    List {
        /// Only show crash logs after this timestamp
        #[arg(long)]
        since: Option<u64>,

        /// Only show crash logs before this timestamp
        #[arg(long)]
        before: Option<u64>,

        /// Filter by bundle identifier
        #[arg(long)]
        bundle_id: Option<String>,

        /// Filter by crash log name
        #[arg(long)]
        name: Option<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Show crash log contents
    Show {
        /// Name of the crash log to display
        name: String,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Delete crash logs
    Delete {
        /// Only delete crash logs after this timestamp
        #[arg(long)]
        since: Option<u64>,

        /// Only delete crash logs before this timestamp
        #[arg(long)]
        before: Option<u64>,

        /// Filter by bundle identifier
        #[arg(long)]
        bundle_id: Option<String>,

        /// Filter by crash log name
        #[arg(long)]
        name: Option<String>,

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

#[derive(Subcommand)]
pub enum SettingsCommands {
    /// Set a device setting
    Set {
        /// Setting name
        name: String,

        /// Setting value
        value: String,

        /// Value type (string, int, bool, etc.)
        #[arg(long)]
        value_type: Option<String>,

        /// Settings domain
        #[arg(long)]
        domain: Option<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// Get a device setting value
    Get {
        /// Setting name
        name: String,

        /// Settings domain
        #[arg(long)]
        domain: Option<String>,

        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },

    /// List available locales
    ListLocale {
        /// Target device/simulator UDID
        #[arg(short, long)]
        udid: Option<String>,
    },
}
