//! HID event generation for iOS.
//!
//! This module provides utilities for generating HID events
//! for touch, tap, swipe, and keyboard input.

pub mod events;

pub use events::{
    button_to_events, key_sequence_to_events, key_to_events, swipe_to_events, tap_to_events,
    text_to_events,
};
