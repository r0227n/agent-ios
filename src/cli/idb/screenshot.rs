use crate::cli::helpers::{with_client, CommandResult, OutputWriter};
use std::io::Write;

pub async fn run(dest_path: String, udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        let image_data = client.screenshot().await?;

        let mut writer = OutputWriter::from_path(&dest_path)?;
        writer.write_all(&image_data)?;
        writer.flush()?;

        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
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
}
