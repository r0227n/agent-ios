//! ADB native protocol connection module.
//!
//! Provides `AdbConnection` that communicates with ADB server (TCP :5037)
//! directly via the `adb_client` crate, eliminating the need for `adb` CLI binary.

use std::net::SocketAddrV4;

use adb_client::{ADBDeviceExt, ADBServer, ADBServerDevice};

use super::commands::{AdbError, Result};

/// Default ADB server address.
const DEFAULT_ADB_ADDR: &str = "127.0.0.1:5037";

/// ADB native protocol connection.
///
/// Wraps `adb_client::ADBServerDevice` for direct TCP communication
/// with the ADB server, replacing `Command::new("adb")` shell-outs.
pub struct AdbConnection {
    device: ADBServerDevice,
}

impl AdbConnection {
    /// Create a connection to a specific device by serial.
    pub fn new(serial: &str) -> Result<Self> {
        let addr: SocketAddrV4 = DEFAULT_ADB_ADDR
            .parse()
            .map_err(|e| AdbError::CommandFailed(format!("Invalid ADB server address: {}", e)))?;
        let device = ADBServerDevice::new(serial.to_string(), Some(addr));
        Ok(Self { device })
    }

    /// Create a connection that auto-detects a single device.
    pub fn autodetect() -> Result<Self> {
        let addr: SocketAddrV4 = DEFAULT_ADB_ADDR
            .parse()
            .map_err(|e| AdbError::CommandFailed(format!("Invalid ADB server address: {}", e)))?;
        let device = ADBServerDevice::autodetect(Some(addr));
        Ok(Self { device })
    }

    /// Create a connection for an optional serial (auto-detect if None).
    pub fn for_device(serial: Option<&str>) -> Result<Self> {
        match serial {
            Some(s) => Self::new(s),
            None => Self::autodetect(),
        }
    }

    /// Execute a shell command and capture stdout as String.
    ///
    /// The command string is split into parts by whitespace.
    /// For commands that need literal spaces in arguments, use `shell_command_args`.
    pub fn shell_command(&mut self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let mut output = Vec::new();
        self.device
            .shell_command(&parts, &mut output)
            .map_err(|e| AdbError::CommandFailed(format!("shell command failed: {}", e)))?;
        Ok(String::from_utf8_lossy(&output).to_string())
    }

    /// Execute a shell command with pre-split arguments.
    pub fn shell_command_args(&mut self, args: &[&str]) -> Result<String> {
        let mut output = Vec::new();
        self.device
            .shell_command(args, &mut output)
            .map_err(|e| AdbError::CommandFailed(format!("shell command failed: {}", e)))?;
        Ok(String::from_utf8_lossy(&output).to_string())
    }

    /// Execute a shell command and capture stdout as raw bytes.
    pub fn shell_command_bytes(&mut self, command: &str) -> Result<Vec<u8>> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let mut output = Vec::new();
        self.device
            .shell_command(&parts, &mut output)
            .map_err(|e| AdbError::CommandFailed(format!("shell command failed: {}", e)))?;
        Ok(output)
    }

    /// Execute a shell command with pre-split args and capture stdout as raw bytes.
    pub fn shell_command_bytes_args(&mut self, args: &[&str]) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        self.device
            .shell_command(args, &mut output)
            .map_err(|e| AdbError::CommandFailed(format!("shell command failed: {}", e)))?;
        Ok(output)
    }

    /// Pull a file from device to a writer.
    pub fn pull<W: std::io::Write>(&mut self, remote_path: &str, writer: &mut W) -> Result<()> {
        self.device
            .pull(&remote_path, writer)
            .map_err(|e| AdbError::CommandFailed(format!("pull failed: {}", e)))
    }

    /// Push data from a reader to a device path.
    pub fn push<R: std::io::Read>(&mut self, reader: R, remote_path: &str) -> Result<()> {
        self.device
            .push(reader, remote_path)
            .map_err(|e| AdbError::CommandFailed(format!("push failed: {}", e)))
    }

    /// Install an APK from a local path.
    pub fn install(&mut self, apk_path: &str) -> Result<()> {
        self.device
            .install(std::path::Path::new(apk_path))
            .map_err(|e| AdbError::CommandFailed(format!("install failed: {}", e)))
    }

    /// Uninstall a package.
    pub fn uninstall(&mut self, package_name: &str) -> Result<()> {
        self.device
            .uninstall(package_name)
            .map_err(|e| AdbError::CommandFailed(format!("uninstall failed: {}", e)))
    }

    /// Capture a screenshot as PNG bytes via framebuffer.
    pub fn framebuffer_bytes(&mut self) -> Result<Vec<u8>> {
        self.device
            .framebuffer_bytes()
            .map_err(|e| AdbError::CommandFailed(format!("framebuffer failed: {}", e)))
    }

    /// Get device logs (logcat) and write to output.
    pub fn get_logs<W: std::io::Write>(&mut self, output: W) -> Result<()> {
        self.device
            .get_logs(output)
            .map_err(|e| AdbError::CommandFailed(format!("logcat failed: {}", e)))
    }

    /// Kill the emulator via `emu kill` command.
    ///
    /// This sends the "emu kill" command to an emulator, which is equivalent
    /// to `adb -s <serial> emu kill`.
    pub fn emu_kill(&mut self) -> Result<()> {
        self.shell_command_args(&["emu", "kill"])?;
        Ok(())
    }

    /// Execute `screenrecord` on device.
    ///
    /// Returns the shell command args that can be used with the device.
    /// Note: screenrecord is a long-running command, so this uses shell_command
    /// which will block until the recording stops (time limit or interrupt).
    pub fn screenrecord(&mut self, remote_path: &str, time_limit: u64) -> Result<()> {
        let time_limit_str = time_limit.to_string();
        self.shell_command_args(&["screenrecord", "--time-limit", &time_limit_str, remote_path])?;
        Ok(())
    }

    /// Get a mutable reference to the inner ADBServerDevice.
    pub fn inner_mut(&mut self) -> &mut ADBServerDevice {
        &mut self.device
    }
}

/// Get an ADB server instance.
fn get_server() -> Result<ADBServer> {
    let addr: SocketAddrV4 = DEFAULT_ADB_ADDR
        .parse()
        .map_err(|e| AdbError::CommandFailed(format!("Invalid ADB server address: {}", e)))?;
    Ok(ADBServer::new(addr))
}

/// Check if ADB server is reachable (native, no CLI).
pub fn is_adb_available() -> bool {
    match get_server() {
        Ok(mut server) => server.version().is_ok(),
        Err(_) => false,
    }
}

/// List connected devices via ADB protocol.
/// Returns a vector of (serial, state) tuples.
pub fn list_devices() -> Result<Vec<(String, String)>> {
    let mut server = get_server()?;
    let devices = server
        .devices()
        .map_err(|e| AdbError::CommandFailed(format!("Failed to list devices: {}", e)))?;
    Ok(devices
        .into_iter()
        .map(|d| (d.identifier, format!("{}", d.state)))
        .collect())
}

/// List available AVDs by reading `~/.android/avd/*.ini` files.
///
/// This replaces `emulator -list-avds` with filesystem reads.
pub fn list_avds() -> Result<Vec<String>> {
    let avd_dir = dirs_avd_path();
    let avd_dir = match avd_dir {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };

    let entries = std::fs::read_dir(&avd_dir).map_err(|e| {
        AdbError::CommandFailed(format!("Failed to read AVD directory {:?}: {}", avd_dir, e))
    })?;

    let mut avds = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("ini") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                // Verify the corresponding .avd directory exists
                let avd_subdir = avd_dir.join(format!("{}.avd", stem));
                if avd_subdir.is_dir() {
                    avds.push(stem.to_string());
                }
            }
        }
    }

    avds.sort();
    Ok(avds)
}

/// Get the AVD directory path (`~/.android/avd/`).
fn dirs_avd_path() -> Option<std::path::PathBuf> {
    // Check ANDROID_AVD_HOME first
    if let Ok(avd_home) = std::env::var("ANDROID_AVD_HOME") {
        let p = std::path::PathBuf::from(avd_home);
        if p.is_dir() {
            return Some(p);
        }
    }

    // Check ANDROID_SDK_HOME (deprecated but still used)
    if let Ok(sdk_home) = std::env::var("ANDROID_SDK_HOME") {
        let p = std::path::PathBuf::from(sdk_home)
            .join(".android")
            .join("avd");
        if p.is_dir() {
            return Some(p);
        }
    }

    // Default: ~/.android/avd/
    std::env::var("HOME")
        .ok()
        .map(|h| std::path::PathBuf::from(h).join(".android").join("avd"))
        .filter(|p| p.is_dir())
}

/// Get a device property via ADB protocol (shell getprop).
pub fn get_prop(serial: &str, prop: &str) -> Result<String> {
    let mut conn = AdbConnection::new(serial)?;
    let command = format!("getprop {}", prop);
    let output = conn.shell_command(&command)?;
    Ok(output.trim().to_string())
}
