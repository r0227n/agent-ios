//! Companion daemon management for iOS.
//!
//! This module handles discovery, spawning, and connection to idb_companion daemons.

pub mod lister;
pub mod resolver;
pub mod spawner;
pub mod state;

pub use lister::CompanionLister;
pub use resolver::CompanionResolver;
pub use state::CompanionState;
