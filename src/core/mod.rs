//! Core Commands - High-level CLI for AI Agents
//!
//! Provides element operations using ref identifiers (@e1, @e2).
//!
//! ## Command List
//! - `tap` - Tap an element
//! - `long-press` - Long press gesture
//! - `fill` - Input into text field (clear + type)
//! - `type` - Append text input
//! - `swipe` - Swipe gesture
//! - `scroll` - Scroll
//! - `get` - Get element information
//! - `is` - Check element state
//! - `wait` - Wait for element
//! - `screenshot` - Take screenshot
//! - `find` - Find elements using semantic locators
//! - `check` - Turn checkbox/switch ON
//! - `uncheck` - Turn checkbox/switch OFF
//! - `select` - Select value from Picker/Spinner

pub mod check;
pub mod fill;
pub mod find;
pub mod get;
pub mod is_cmd;
pub mod long_press;
pub mod ref_resolver;
pub mod screenshot;
pub mod scroll;
pub mod select;
pub mod swipe;
pub mod tap;
pub mod text_input;
pub mod type_cmd;
pub mod wait;
