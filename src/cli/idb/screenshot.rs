use crate::companion::CompanionResolver;
use std::io::Write;

/// Check if the destination path indicates stdout output
fn is_stdout_output(dest_path: &str) -> bool {
    dest_path == "-"
}

pub async fn run(
    dest_path: String,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Connect to companion (with auto-spawning if needed)
    let resolver = CompanionResolver::new();
    let mut client = resolver.connect(udid.as_deref()).await?;

    // 2. Take screenshot
    let image_data = client.screenshot().await?;

    // 3. Write to file or stdout
    if is_stdout_output(&dest_path) {
        // Write to stdout (binary mode)
        std::io::stdout().write_all(&image_data)?;
        std::io::stdout().flush()?;
    } else {
        // Write to file (atomic write)
        std::fs::write(&dest_path, &image_data)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_stdout_output_with_dash() {
        assert!(is_stdout_output("-"));
    }

    #[test]
    fn test_is_stdout_output_with_file_path() {
        assert!(!is_stdout_output("/tmp/screenshot.png"));
    }

    #[test]
    fn test_is_stdout_output_with_relative_path() {
        assert!(!is_stdout_output("screenshot.png"));
    }

    #[test]
    fn test_is_stdout_output_with_dash_in_path() {
        assert!(!is_stdout_output("/tmp/my-screenshot.png"));
    }
}
