//! HID (Human Interface Device) commands.
//!
//! Provides low-level input simulation:
//! - `button` - Press hardware buttons (home, lock, etc.)
//! - `key` - Press individual keys by keycode
//! - `key-sequence` - Press a sequence of keys

pub mod button;
pub mod key;
pub mod key_sequence;
