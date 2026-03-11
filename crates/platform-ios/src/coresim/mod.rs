//! CoreSimulator-backed iOS simulator helpers.
//!
//! This module exposes lightweight wrappers around the private CoreSimulator
//! framework for app install, launch, terminate, and booted-device discovery.

mod device;
mod error;

use std::path::Path;

pub use device::BootedDevice;
pub use error::CoreSimError;

/// Install an application (.app bundle) onto the specified simulator.
pub fn install_app(udid: &str, app_path: &Path) -> Result<(), CoreSimError> {
    device::install_application(udid, app_path)
}

/// Launch an application on the specified simulator. Returns the PID.
pub fn launch_app(udid: &str, bundle_id: &str) -> Result<u32, CoreSimError> {
    device::launch_application(udid, bundle_id)
}

/// Terminate a running application on the specified simulator.
pub fn terminate_app(udid: &str, bundle_id: &str) -> Result<(), CoreSimError> {
    device::terminate_application(udid, bundle_id)
}

/// Find the first booted simulator device.
pub fn get_booted_device() -> Result<BootedDevice, CoreSimError> {
    device::find_booted_device()
}
