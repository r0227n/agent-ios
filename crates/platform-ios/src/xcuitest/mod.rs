//! XCUITest Runner HTTP client and lifecycle primitives.

/// HTTP client for talking to the runner server.
pub mod client;
/// Build and process management for the runner app bundles.
pub mod runner;
/// Request and response payload types used by the runner API.
pub mod types;

pub use client::XCUITestClient;
pub use runner::{
    build_for_testing, ensure_runner_started, start_runner_detached, RunnerBuildProducts,
    RunnerStartError, XCUITestRunner,
};
