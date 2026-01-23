//! Installation-related type definitions.

use serde::{Deserialize, Serialize};

/// Result of an install operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledArtifact {
    pub name: String,
    pub uuid: Option<String>,
    pub progress: Option<f64>,
}

/// Compression type for install payloads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    Gzip,
    Zstd,
}

impl std::str::FromStr for Compression {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GZIP" => Ok(Self::Gzip),
            "ZSTD" => Ok(Self::Zstd),
            _ => Err(format!(
                "Invalid compression type: {}. Use GZIP or ZSTD.",
                s
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_from_str_gzip() {
        let comp: Compression = "GZIP".parse().unwrap();
        assert_eq!(comp, Compression::Gzip);
    }

    #[test]
    fn test_compression_from_str_gzip_lowercase() {
        let comp: Compression = "gzip".parse().unwrap();
        assert_eq!(comp, Compression::Gzip);
    }

    #[test]
    fn test_compression_from_str_zstd() {
        let comp: Compression = "ZSTD".parse().unwrap();
        assert_eq!(comp, Compression::Zstd);
    }

    #[test]
    fn test_compression_from_str_zstd_lowercase() {
        let comp: Compression = "zstd".parse().unwrap();
        assert_eq!(comp, Compression::Zstd);
    }

    #[test]
    fn test_compression_from_str_invalid() {
        let result: Result<Compression, _> = "invalid".parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid compression type"));
    }

    #[test]
    fn test_compression_from_str_mixed_case() {
        let comp: Compression = "GzIp".parse().unwrap();
        assert_eq!(comp, Compression::Gzip);
    }
}
