use crate::cli::helpers::{with_client, CommandResult, OutputWriter};
use std::io::Write;
use std::time::Duration;
use tokio::time::sleep;

/// Maximum number of retry attempts for screenshot
const MAX_RETRIES: u32 = 5;

/// Initial retry delay in milliseconds
const INITIAL_RETRY_DELAY_MS: u64 = 1000;

pub async fn run(dest_path: String, udid: Option<String>) -> CommandResult {
    let mut attempt = 0;

    loop {
        attempt += 1;

        match try_screenshot(&dest_path, udid.as_deref()).await {
            Ok(()) => return Ok(()),
            Err(e) if should_retry(e.as_ref(), attempt) => {
                // Exponential backoff: 500ms, 1000ms, 2000ms
                let delay_ms = INITIAL_RETRY_DELAY_MS * (1 << (attempt - 1));
                eprintln!(
                    "Screenshot attempt {} failed (framebuffer not ready), retrying in {}ms...",
                    attempt, delay_ms
                );
                sleep(Duration::from_millis(delay_ms)).await;
            }
            Err(e) => {
                // Non-retryable error or max retries exceeded
                if is_framebuffer_error(e.as_ref()) && attempt >= MAX_RETRIES {
                    return Err(format!(
                        "Screenshot failed after {} attempts: {}\n\
                        \n\
                        The simulator's framebuffer may not be initialized yet. Try:\n\
                        1. Interacting with the simulator UI\n\
                        2. Running 'idb ui describe-all' first\n\
                        3. Waiting a few seconds after booting the simulator",
                        MAX_RETRIES, e
                    )
                    .into());
                }
                return Err(e);
            }
        }
    }
}

async fn try_screenshot(dest_path: &str, udid: Option<&str>) -> CommandResult {
    with_client(udid, |mut client| async move {
        let image_data = client.screenshot().await?;

        let mut writer = OutputWriter::from_path(dest_path)?;
        writer.write_all(&image_data)?;
        writer.flush()?;

        Ok(())
    })
    .await
}

/// Determine if an error is retryable (framebuffer not ready)
fn should_retry(error: &dyn std::error::Error, attempt: u32) -> bool {
    if attempt >= MAX_RETRIES {
        return false;
    }

    is_framebuffer_error(error)
}

/// Check if error is related to framebuffer initialization
fn is_framebuffer_error(error: &dyn std::error::Error) -> bool {
    let error_msg = error.to_string();
    error_msg.contains("No Image available to encode")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::helpers::OutputWriter;

    #[test]
    fn test_is_stdout_output_with_dash() {
        let writer = OutputWriter::from_path("-").unwrap();
        assert!(writer.is_stdout());
    }

    #[test]
    fn test_is_stdout_output_with_file_path() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let writer = OutputWriter::from_path(temp_file.path().to_str().unwrap()).unwrap();
        assert!(!writer.is_stdout());
    }

    #[test]
    fn test_framebuffer_error_detection() {
        let error: Box<dyn std::error::Error + Send + Sync> = "No Image available to encode".into();
        assert!(is_framebuffer_error(error.as_ref()));

        let error: Box<dyn std::error::Error + Send + Sync> = "Some other error".into();
        assert!(!is_framebuffer_error(error.as_ref()));
    }
}
