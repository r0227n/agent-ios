//! Type definitions for agent-mobile.
//!
//! This module contains all shared types used across the codebase including
//! target descriptions, addresses, compression types, and installation artifacts.

mod install;
mod output;
mod target;

pub use install::{Compression, InstalledArtifact};
pub use output::{human_format_target, json_format_target};
pub use target::{Address, CompanionInfo, DeviceInfo, TargetType};

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Platform Types
// ============================================================================

/// Supported mobile platforms.
///
/// # Example
///
/// ```
/// use agent_mobile_core::Platform;
///
/// let ios: Platform = "ios".parse().unwrap();
/// assert_eq!(ios.as_str(), "ios");
///
/// let android: Platform = "android".parse().unwrap();
/// assert_eq!(android.as_str(), "android");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    #[cfg_attr(feature = "cli", value(name = "ios"))]
    Ios,
    #[cfg_attr(feature = "cli", value(name = "android"))]
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
///
/// # Example
///
/// ```
/// use agent_mobile_core::ScrollDirection;
///
/// let dir: ScrollDirection = "up".parse().unwrap();
/// assert_eq!(dir.as_str(), "up");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum ScrollDirection {
    #[cfg_attr(feature = "cli", value(name = "up"))]
    Up,
    #[cfg_attr(feature = "cli", value(name = "down"))]
    Down,
    #[cfg_attr(feature = "cli", value(name = "left"))]
    Left,
    #[cfg_attr(feature = "cli", value(name = "right"))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_from_str() {
        assert_eq!("ios".parse::<Platform>().unwrap(), Platform::Ios);
        assert_eq!("android".parse::<Platform>().unwrap(), Platform::Android);
        assert!("invalid".parse::<Platform>().is_err());
    }

    #[test]
    fn test_scroll_direction_from_str() {
        assert_eq!(
            "up".parse::<ScrollDirection>().unwrap(),
            ScrollDirection::Up
        );
        assert_eq!(
            "down".parse::<ScrollDirection>().unwrap(),
            ScrollDirection::Down
        );
        assert!("invalid".parse::<ScrollDirection>().is_err());
    }
}
