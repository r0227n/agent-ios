//! Type definitions for the doctor command.
//!
//! Provides common types for environment checks across platforms.

use serde::Serialize;

/// Status of a single check.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    /// Check passed successfully.
    Ok,
    /// Check failed with an error.
    Error,
}

impl CheckStatus {
    /// Returns true if the status is OK.
    pub fn is_ok(&self) -> bool {
        matches!(self, CheckStatus::Ok)
    }
}

/// Result of a single environment check.
#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    /// Name of the check (e.g., "xcode_cli", "adb").
    pub name: String,
    /// Status of the check.
    pub status: CheckStatus,
    /// Version string if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Path if relevant (e.g., ANDROID_HOME).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Error message if check failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Installation hint if check failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl CheckResult {
    /// Create a successful check result.
    pub fn ok(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Ok,
            version: None,
            path: None,
            message: None,
            hint: None,
        }
    }

    /// Create a successful check result with version.
    pub fn ok_with_version(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Ok,
            version: Some(version.into()),
            path: None,
            message: None,
            hint: None,
        }
    }

    /// Create a successful check result with path.
    pub fn ok_with_path(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Ok,
            version: None,
            path: Some(path.into()),
            message: None,
            hint: None,
        }
    }

    /// Create a failed check result.
    pub fn error(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Error,
            version: None,
            path: None,
            message: Some(message.into()),
            hint: None,
        }
    }

    /// Add a hint to this check result.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    /// Add a path to this check result.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
}

/// Results for a platform.
#[derive(Debug, Clone, Serialize)]
pub struct PlatformResults {
    /// List of check results.
    pub checks: Vec<CheckResult>,
}

impl PlatformResults {
    /// Create a new empty platform results.
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    /// Add a check result.
    pub fn add(&mut self, result: CheckResult) {
        self.checks.push(result);
    }

    /// Count passed checks.
    pub fn passed_count(&self) -> usize {
        self.checks.iter().filter(|c| c.status.is_ok()).count()
    }

    /// Count failed checks.
    pub fn failed_count(&self) -> usize {
        self.checks.iter().filter(|c| !c.status.is_ok()).count()
    }
}

impl Default for PlatformResults {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete doctor results.
#[derive(Debug, Clone, Serialize)]
pub struct DoctorResults {
    /// iOS check results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ios: Option<PlatformResults>,
    /// Android check results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub android: Option<PlatformResults>,
    /// Summary of all checks.
    pub summary: Summary,
}

/// Summary of check results.
#[derive(Debug, Clone, Serialize)]
pub struct Summary {
    /// Number of passed checks.
    pub passed: usize,
    /// Number of failed checks.
    pub failed: usize,
}
