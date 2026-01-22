//! Type definitions for agent-mobile.
//!
//! This module re-exports types from the core module for backward compatibility.
//! New code should use `crate::core::types` directly.

use serde::{Deserialize, Serialize};
use std::fmt;

// Keep the old modules for tests and backward compatibility
pub mod install;
pub mod merge;
pub mod output;
pub mod target;

// Re-export from the old modules (for backward compatibility with existing code)
pub use install::{Compression, InstalledArtifact};
pub use merge::merge_connected_targets;
pub use output::{human_format_target, json_format_target};
pub use target::{Address, CompanionInfo, TargetDescription, TargetType};

// ============================================================================
// Platform Types
// ============================================================================

/// Supported mobile platforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Ios,
    Android,
}

impl Platform {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ios => "ios",
            Self::Android => "android",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ios" => Ok(Self::Ios),
            "android" => Ok(Self::Android),
            _ => Err(format!("Invalid platform: {}. Use 'ios' or 'android'.", s)),
        }
    }
}

// ============================================================================
// Gesture Types
// ============================================================================

/// Direction for scroll/swipe gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

impl ScrollDirection {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    /// Get swipe coordinates for this direction (assuming center origin).
    /// Returns (start_offset, end_offset) where offset is (dx, dy) from center.
    pub fn to_swipe_offsets(self, distance: f64) -> ((f64, f64), (f64, f64)) {
        let half = distance / 2.0;
        match self {
            // Swipe up = finger moves from bottom to top
            Self::Up => ((0.0, half), (0.0, -half)),
            // Swipe down = finger moves from top to bottom
            Self::Down => ((0.0, -half), (0.0, half)),
            // Swipe left = finger moves from right to left
            Self::Left => ((half, 0.0), (-half, 0.0)),
            // Swipe right = finger moves from left to right
            Self::Right => ((-half, 0.0), (half, 0.0)),
        }
    }
}

impl fmt::Display for ScrollDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ScrollDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "up" => Ok(Self::Up),
            "down" => Ok(Self::Down),
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            _ => Err(format!(
                "Invalid scroll direction: {}. Use 'up', 'down', 'left', or 'right'.",
                s
            )),
        }
    }
}
