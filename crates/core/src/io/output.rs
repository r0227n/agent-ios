//! Output destination helpers for commands
//!
//! This module provides utilities for handling output to
//! stdout or files, commonly used by screenshot and file pull commands.

use std::fs::File;
use std::io::{self, Write};

/// Abstraction for output destination (stdout or file)
pub enum OutputWriter {
    /// Write to stdout
    Stdout(io::Stdout),
    /// Write to a file
    File(File),
    /// Write to both file and stdout (tee mode)
    Tee { file: File, stdout: io::Stdout },
}

impl OutputWriter {
    /// Create a writer based on the path.
    ///
    /// If path is "-", returns stdout writer.
    /// Otherwise, creates a file at the given path.
    ///
    /// # Example
    ///
    /// ```
    /// use agent_mobile_core::OutputWriter;
    /// use std::io::Write;
    ///
    /// // "-" creates a stdout writer
    /// let mut writer = OutputWriter::from_path("-").unwrap();
    /// assert!(writer.is_stdout());
    /// ```
    pub fn from_path(path: &str) -> io::Result<Self> {
        if path == "-" {
            Ok(Self::Stdout(io::stdout()))
        } else {
            Ok(Self::File(File::create(path)?))
        }
    }

    /// Create a writer that outputs to both file and stdout (tee mode).
    ///
    /// If path is "-", returns stdout-only writer.
    /// Otherwise, creates a file and also writes to stdout.
    pub fn tee_from_path(path: &str) -> io::Result<Self> {
        if path == "-" {
            Ok(Self::Stdout(io::stdout()))
        } else {
            Ok(Self::Tee {
                file: File::create(path)?,
                stdout: io::stdout(),
            })
        }
    }

    /// Check if output is to stdout
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_stdout(&self) -> bool {
        matches!(self, Self::Stdout(_))
    }
}

impl Write for OutputWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Self::Stdout(s) => s.write(buf),
            Self::File(f) => f.write(buf),
            Self::Tee { file, stdout } => {
                // Write to both using write_all to avoid partial writes
                stdout.write_all(buf)?;
                file.write_all(buf)?;
                Ok(buf.len())
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Stdout(s) => s.flush(),
            Self::File(f) => f.flush(),
            Self::Tee { file, stdout } => {
                stdout.flush()?;
                file.flush()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_output_writer_stdout() {
        let writer = OutputWriter::from_path("-").unwrap();
        assert!(writer.is_stdout());
    }

    #[test]
    fn test_output_writer_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        let writer = OutputWriter::from_path(path).unwrap();
        assert!(!writer.is_stdout());
    }

    #[test]
    fn test_output_writer_write_to_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        {
            let mut writer = OutputWriter::from_path(path).unwrap();
            writer.write_all(b"test data").unwrap();
            writer.flush().unwrap();
        }

        let contents = std::fs::read_to_string(path).unwrap();
        assert_eq!(contents, "test data");
    }
}
