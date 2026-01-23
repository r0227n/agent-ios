//! Snapshot types and utilities for UI element extraction.

mod traits;
pub mod types;

pub use traits::extract_traits_for_type;
pub use types::{is_interactive_type, Frame, RawElement};
