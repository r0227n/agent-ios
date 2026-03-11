use std::ffi::c_void;
use std::path::Path;
use std::sync::OnceLock;

use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::{AnyClass, AnyObject};
use objc2_foundation::{NSError, NSString, NSURL};

use super::error::CoreSimError;

// dlopen/dlsym FFI
extern "C" {
    fn dlopen(filename: *const i8, flags: i32) -> *mut c_void;
}
const RTLD_LAZY: i32 = 0x1;
const RTLD_GLOBAL: i32 = 0x8;

/// CoreSimulator SimDeviceState constants
/// See: CoreSimulator.framework/Headers/SimDevice.h
#[allow(dead_code)]
const SIM_DEVICE_STATE_CREATING: u64 = 0;
#[allow(dead_code)]
const SIM_DEVICE_STATE_SHUTDOWN: u64 = 1;
#[allow(dead_code)]
const SIM_DEVICE_STATE_BOOTING: u64 = 2;
const SIM_DEVICE_STATE_BOOTED: u64 = 3;
#[allow(dead_code)]
const SIM_DEVICE_STATE_SHUTTING_DOWN: u64 = 4;

static LOAD_FRAMEWORK: OnceLock<bool> = OnceLock::new();

/// Load CoreSimulator.framework via dlopen.
///
/// Must be called before any other CoreSimulator FFI operations.
/// Safe to call multiple times — the result is cached after the first attempt.
pub(crate) fn load_framework() -> Result<(), CoreSimError> {
    let loaded = LOAD_FRAMEWORK.get_or_init(|| {
        let paths = [
            "/Applications/Xcode.app/Contents/Developer/Library/PrivateFrameworks/CoreSimulator.framework/CoreSimulator",
            "/Applications/Xcode-beta.app/Contents/Developer/Library/PrivateFrameworks/CoreSimulator.framework/CoreSimulator",
            "/Library/Developer/PrivateFrameworks/CoreSimulator.framework/CoreSimulator",
        ];

        let found = paths.iter().any(|path| {
            let c_path = match std::ffi::CString::new(*path) {
                Ok(p) => p,
                Err(_) => return false,
            };
            let handle = unsafe { dlopen(c_path.as_ptr(), RTLD_LAZY | RTLD_GLOBAL) };
            !handle.is_null()
        });

        if found {
            return true;
        }

        // Try xcode-select -p to find the developer directory
        if let Ok(output) = std::process::Command::new("xcode-select")
            .arg("-p")
            .output()
        {
            if output.status.success() {
                let dev_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let path = format!(
                    "{}/Library/PrivateFrameworks/CoreSimulator.framework/CoreSimulator",
                    dev_dir
                );
                if let Ok(c_path) = std::ffi::CString::new(path) {
                    let handle =
                        unsafe { dlopen(c_path.as_ptr(), RTLD_LAZY | RTLD_GLOBAL) };
                    if !handle.is_null() {
                        return true;
                    }
                }
            }
        }

        false
    });

    if *loaded {
        Ok(())
    } else {
        Err(CoreSimError::FrameworkNotFound)
    }
}

/// Booted device information returned from CoreSimulator.
pub struct BootedDevice {
    /// Simulator UDID.
    pub udid: String,
    /// Human-readable simulator name.
    pub name: String,
}

/// Get a SimServiceContext and the default device set, returning the devices array.
fn get_device_set() -> Result<Retained<AnyObject>, CoreSimError> {
    load_framework()?;

    let cls = AnyClass::get(c"SimServiceContext")
        .ok_or_else(|| CoreSimError::FfiError("SimServiceContext class not found".into()))?;

    // +[SimServiceContext sharedServiceContextForDeveloperDir:error:]
    let context: Result<Retained<AnyObject>, Retained<NSError>> = unsafe {
        let dir: *const AnyObject = std::ptr::null();
        msg_send![cls, sharedServiceContextForDeveloperDir: dir, error: _]
    };
    let context = context.map_err(|e| CoreSimError::FfiError(format!("{}", e)))?;

    // -[SimServiceContext defaultDeviceSetWithError:]
    let device_set: Result<Retained<AnyObject>, Retained<NSError>> =
        unsafe { msg_send![&*context, defaultDeviceSetWithError: _] };
    device_set.map_err(|e| CoreSimError::FfiError(format!("{}", e)))
}

/// Get the devices array from a device set and its count.
fn get_devices_from_set(
    device_set: &AnyObject,
) -> Result<(Retained<AnyObject>, usize), CoreSimError> {
    // -[SimDeviceSet devices] -> NSArray
    let devices: Retained<AnyObject> = unsafe { msg_send![device_set, devices] };
    let count: usize = unsafe { msg_send![&*devices, count] };
    Ok((devices, count))
}

/// Get a device at a specific index from an NSArray.
fn device_at_index(devices: &AnyObject, index: usize) -> Retained<AnyObject> {
    unsafe { msg_send![devices, objectAtIndex: index] }
}

/// Get the UDID string of a SimDevice.
fn device_udid_string(device: &AnyObject) -> String {
    let udid_obj: Retained<AnyObject> = unsafe { msg_send![device, UDID] };
    let udid_str: Retained<NSString> = unsafe { msg_send![&*udid_obj, UUIDString] };
    udid_str.to_string()
}

/// Find the first booted simulator via CoreSimulator APIs.
pub(crate) fn find_booted_device() -> Result<BootedDevice, CoreSimError> {
    let device_set = get_device_set()?;
    let (devices, count) = get_devices_from_set(&device_set)?;

    for i in 0..count {
        let device = device_at_index(&devices, i);

        let state: u64 = unsafe { msg_send![&*device, state] };
        if state == SIM_DEVICE_STATE_BOOTED {
            let udid = device_udid_string(&device);
            let name_ns: Retained<NSString> = unsafe { msg_send![&*device, name] };

            return Ok(BootedDevice {
                udid,
                name: name_ns.to_string(),
            });
        }
    }

    Err(CoreSimError::NoBootedDevice)
}

/// Find a specific simulator device by UDID.
fn find_device_by_udid(udid: &str) -> Result<Retained<AnyObject>, CoreSimError> {
    let device_set = get_device_set()?;
    let (devices, count) = get_devices_from_set(&device_set)?;

    let target_udid = udid.to_uppercase();

    for i in 0..count {
        let device = device_at_index(&devices, i);

        if device_udid_string(&device).to_uppercase() == target_udid {
            return Ok(device);
        }
    }

    Err(CoreSimError::DeviceNotFound(udid.to_string()))
}

/// Install an application on the specified simulator.
pub(crate) fn install_application(udid: &str, app_path: &Path) -> Result<(), CoreSimError> {
    let device = find_device_by_udid(udid)?;

    let path_str = app_path
        .to_str()
        .ok_or_else(|| CoreSimError::InstallFailed("Invalid app path encoding".into()))?;
    let ns_path = NSString::from_str(path_str);

    let url_cls = AnyClass::get(c"NSURL")
        .ok_or_else(|| CoreSimError::FfiError("NSURL class not found".into()))?;
    let url: Retained<NSURL> = unsafe { msg_send![url_cls, fileURLWithPath: &*ns_path] };

    let dict_cls = AnyClass::get(c"NSDictionary")
        .ok_or_else(|| CoreSimError::FfiError("NSDictionary class not found".into()))?;
    let options: Retained<AnyObject> = unsafe { msg_send![dict_cls, dictionary] };

    let result: Result<(), Retained<NSError>> =
        unsafe { msg_send![&*device, installApplication: &*url, withOptions: &*options, error: _] };

    result.map_err(|e| CoreSimError::InstallFailed(format!("{}", e)))
}

/// Launch an application on the specified simulator. Returns the PID.
pub(crate) fn launch_application(udid: &str, bundle_id: &str) -> Result<u32, CoreSimError> {
    let device = find_device_by_udid(udid)?;

    let ns_bundle_id = NSString::from_str(bundle_id);

    let dict_cls = AnyClass::get(c"NSDictionary")
        .ok_or_else(|| CoreSimError::FfiError("NSDictionary class not found".into()))?;
    let options: Retained<AnyObject> = unsafe { msg_send![dict_cls, dictionary] };

    let mut pid: i32 = 0;
    let result: Result<(), Retained<NSError>> = unsafe {
        msg_send![
            &*device,
            launchApplicationWithID: &*ns_bundle_id,
            options: &*options,
            pid: &mut pid as *mut i32,
            error: _
        ]
    };

    result.map_err(|e| CoreSimError::LaunchFailed(format!("{}", e)))?;

    if pid < 0 {
        return Err(CoreSimError::LaunchFailed(format!(
            "invalid PID returned: {}",
            pid
        )));
    }

    Ok(pid as u32)
}

/// Terminate an application on the specified simulator.
pub(crate) fn terminate_application(udid: &str, bundle_id: &str) -> Result<(), CoreSimError> {
    let device = find_device_by_udid(udid)?;

    let ns_bundle_id = NSString::from_str(bundle_id);

    let result: Result<(), Retained<NSError>> =
        unsafe { msg_send![&*device, terminateApplicationWithID: &*ns_bundle_id, error: _] };

    result.map_err(|e| CoreSimError::TerminateFailed(format!("{}", e)))
}
