//! screenshot command - Capture screenshot
//!
//! ```bash
//! agent-mobile screenshot --output output.png
//! agent-mobile screenshot --output /tmp/ --format jpeg
//! ```

use clap::Args;

use agent_mobile_core::Platform;

use crate::helpers::client::CommandResult;
use crate::helpers::common_args::DeviceArgs;

use agent_mobile_gateway::DeviceResolver;

/// Supported image formats for screenshots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
}

impl ImageFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
        }
    }

    /// Check if format is supported on Android
    pub fn is_android_supported(&self) -> bool {
        matches!(self, Self::Png)
    }
}

impl std::str::FromStr for ImageFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "png" => Ok(Self::Png),
            "jpg" | "jpeg" => Ok(Self::Jpeg),
            _ => Err(format!(
                "Unsupported image format: {}. Use 'png' or 'jpeg'.",
                s
            )),
        }
    }
}

/// Arguments for the screenshot command
#[derive(Args, Debug)]
pub struct ScreenshotArgs {
    /// Output file or directory path
    #[arg(
        short = 'o',
        long,
        long_help = "Output file or directory path\n\
                     \n\
                     Modes:\n  \
                     • If path ends with '/': treat as directory and auto-generate filename\n  \
                     • Otherwise: treat as full file path\n  \
                     • If omitted: current directory with timestamp filename"
    )]
    pub output: Option<String>,

    /// Image format (png or jpeg)
    #[arg(short = 'f', long, default_value = "png")]
    pub format: ImageFormat,

    #[command(flatten)]
    pub device: DeviceArgs,
}

/// Generate timestamp-based filename
fn generate_filename(format: ImageFormat) -> String {
    let now = chrono::Local::now();
    format!(
        "screenshot_{}.{}",
        now.format("%Y%m%d_%H%M%S"),
        format.extension()
    )
}

/// Validate output path
///
/// Returns Ok(()) if valid, Err with error message if invalid.
///
/// Valid paths:
/// - Paths containing '/', '\', or '.' (file or directory paths)
/// - Directory names that are not known format names
///
/// Invalid paths:
/// - Empty strings
/// - Known format names (json, base64, png, jpg, jpeg) without path separators
/// - Paths with invalid characters (<, >, |, \0, \n, \r)
fn validate_output_path(output: &str) -> Result<(), String> {
    // Empty string
    if output.trim().is_empty() {
        return Err("Output path cannot be empty".to_string());
    }

    // Known format names that should be rejected if used alone
    const KNOWN_FORMATS: &[&str] = &["json", "base64", "png", "jpg", "jpeg", "yaml"];

    // Check for known format names without path separators
    if !output.contains('/') && !output.contains('\\') && !output.contains('.') {
        let lower = output.to_lowercase();
        if KNOWN_FORMATS.contains(&lower.as_str()) {
            return Err(format!(
                "Invalid output path: '{}'. Did you mean to use a file path?\n\
                Use '--output /path/to/file.png' for file output.",
                output
            ));
        }
    }

    // Check for invalid path characters
    let invalid_chars = ['<', '>', '|', '\0', '\n', '\r'];
    for ch in invalid_chars {
        if output.contains(ch) {
            return Err(format!(
                "Invalid character '{}' in output path",
                ch.escape_default()
            ));
        }
    }

    Ok(())
}

/// Resolve output path based on user input
///
/// Logic:
/// 1. If `output` is None: current directory + timestamp filename
/// 2. If `output` ends with '/': directory + timestamp filename
/// 3. Otherwise: treat as full file path
fn resolve_output_path(
    output: Option<&str>,
    format: ImageFormat,
) -> Result<String, std::io::Error> {
    match output {
        None => {
            // Default: current directory + timestamp filename
            Ok(generate_filename(format))
        }
        Some(out) => {
            // Validate input first
            validate_output_path(out)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

            // Check if it's a directory path
            if out.ends_with('/') || out.ends_with(std::path::MAIN_SEPARATOR) {
                // Directory specified, generate filename
                let filename = generate_filename(format);
                return Ok(format!("{}{}", out, filename));
            }

            // Check if path exists and is a directory
            let path_obj = std::path::Path::new(out);
            if path_obj.exists() && path_obj.is_dir() {
                let filename = generate_filename(format);
                return Ok(path_obj.join(filename).to_string_lossy().to_string());
            }

            // Treat as full file path
            Ok(out.to_string())
        }
    }
}

/// Execute the screenshot command
pub async fn run(args: ScreenshotArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };
    let resolved_path = resolve_output_path(args.output.as_deref(), args.format)?;

    match platform {
        Platform::Ios => {
            execute_screenshot_ios(args.device.udid.as_deref(), &resolved_path, args.format).await
        }
        Platform::Android => {
            execute_screenshot_android(args.device.udid.as_deref(), &resolved_path, args.format)
                .await
        }
    }
}

/// Execute screenshot on iOS using XCUITest Runner.
async fn execute_screenshot_ios(
    _udid: Option<&str>,
    path: &str,
    format: ImageFormat,
) -> CommandResult {
    use crate::helpers::client::with_xcuitest;

    let png_bytes = with_xcuitest(|client| async move { client.screenshot().await }).await?;

    let final_bytes = match format {
        ImageFormat::Png => png_bytes,
        ImageFormat::Jpeg => convert_png_to_jpeg(&png_bytes)?,
    };

    std::fs::write(path, &final_bytes)?;
    println!("Screenshot saved to: {}", path);

    Ok(())
}

/// Convert PNG bytes to JPEG bytes.
fn convert_png_to_jpeg(
    png_bytes: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let img = image::load_from_memory_with_format(png_bytes, image::ImageFormat::Png)?;
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Jpeg)?;
    Ok(buf.into_inner())
}

/// Execute screenshot on Android
async fn execute_screenshot_android(
    udid: Option<&str>,
    path: &str,
    format: ImageFormat,
) -> CommandResult {
    use tokio::process::Command;

    // Validate format support on Android - only PNG is supported
    if !format.is_android_supported() {
        return Err(
            "Android only supports PNG format. JPEG conversion is not implemented. \
            Please use '--format png' or omit the format option."
                .to_string()
                .into(),
        );
    }

    // Use adb to capture screenshot
    let serial_arg = udid
        .map(|s| vec!["-s".to_string(), s.to_string()])
        .unwrap_or_default();

    let output = Command::new("adb")
        .args(&serial_arg)
        .args(["exec-out", "screencap", "-p"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("adb screencap failed: {}", stderr).into());
    }

    let bytes = output.stdout;

    // File mode
    std::fs::write(path, &bytes)?;
    println!("Screenshot saved to: {}", path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // Tests for validate_output_path()

    #[test]
    fn test_validate_output_path_valid_file() {
        assert!(validate_output_path("/tmp/test.png").is_ok());
        assert!(validate_output_path("./output.png").is_ok());
        assert!(validate_output_path("test.png").is_ok());
    }

    #[test]
    fn test_validate_output_path_valid_directory() {
        assert!(validate_output_path("/tmp/").is_ok());
        assert!(validate_output_path("./dir/").is_ok());
    }

    #[test]
    fn test_validate_output_path_empty() {
        assert!(validate_output_path("").is_err());
        assert!(validate_output_path("  ").is_err());
    }

    #[test]
    fn test_validate_output_path_format_like() {
        // Known format names should error
        assert!(validate_output_path("json").is_err());
        assert!(validate_output_path("base64").is_err());
        assert!(validate_output_path("png").is_err());
        assert!(validate_output_path("jpeg").is_err());
        assert!(validate_output_path("jpg").is_err());
        assert!(validate_output_path("yaml").is_err());
        // Unknown names are allowed as directories
        assert!(validate_output_path("mydir").is_ok());
    }

    #[test]
    fn test_validate_output_path_invalid_chars() {
        assert!(validate_output_path("test|file.png").is_err());
        assert!(validate_output_path("test<file.png").is_err());
        assert!(validate_output_path("test>file.png").is_err());
    }

    // Tests for ImageFormat

    #[test]
    fn test_image_format_from_str() {
        assert_eq!(ImageFormat::from_str("png").unwrap(), ImageFormat::Png);
        assert_eq!(ImageFormat::from_str("PNG").unwrap(), ImageFormat::Png);
        assert_eq!(ImageFormat::from_str("jpeg").unwrap(), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_str("jpg").unwrap(), ImageFormat::Jpeg);
        assert!(ImageFormat::from_str("gif").is_err());
    }

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpeg");
    }

    #[test]
    fn test_image_format_android_support() {
        assert!(ImageFormat::Png.is_android_supported());
        assert!(!ImageFormat::Jpeg.is_android_supported());
    }

    // Tests for resolve_output_path()

    #[test]
    fn test_resolve_output_path_none() {
        let result = resolve_output_path(None, ImageFormat::Png);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.starts_with("screenshot_"));
        assert!(path.ends_with(".png"));
    }

    #[test]
    fn test_resolve_output_path_directory_with_slash() {
        let result = resolve_output_path(Some("/tmp/"), ImageFormat::Png);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.starts_with("/tmp/screenshot_"));
        assert!(path.ends_with(".png"));
    }

    #[test]
    fn test_resolve_output_path_full_path() {
        let result = resolve_output_path(Some("/tmp/test.png"), ImageFormat::Png);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/tmp/test.png");
    }

    #[test]
    fn test_resolve_output_path_invalid_format_like() {
        let result = resolve_output_path(Some("json"), ImageFormat::Png);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_output_path_invalid_chars() {
        let result = resolve_output_path(Some("test|file.png"), ImageFormat::Png);
        assert!(result.is_err());
    }

    // Tests for generate_filename()

    #[test]
    fn test_generate_filename_format() {
        let filename = generate_filename(ImageFormat::Png);
        assert!(filename.starts_with("screenshot_"));
        assert!(filename.ends_with(".png"));
        assert!(filename.contains("_")); // timestamp separator
    }

    #[test]
    fn test_generate_filename_jpeg() {
        let filename = generate_filename(ImageFormat::Jpeg);
        assert!(filename.starts_with("screenshot_"));
        assert!(filename.ends_with(".jpeg"));
    }
}
