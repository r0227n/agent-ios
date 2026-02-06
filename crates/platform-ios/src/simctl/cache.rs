//! Device list cache for reducing repeated simctl invocations.
//!
//! Caches the result of `list_simulators()` with a fixed TTL of 5 seconds.
//! The cache is automatically invalidated by lifecycle operations (boot, shutdown, etc.).

use std::sync::Mutex;
use std::time::{Duration, Instant};

use agent_mobile_core::types::DeviceInfo;

/// Global device cache instance.
static DEVICE_CACHE: Mutex<CacheInner> = Mutex::new(CacheInner::new());

struct CacheInner {
    devices: Option<Vec<DeviceInfo>>,
    fetched_at: Option<Instant>,
}

impl CacheInner {
    const TTL: Duration = Duration::from_secs(5);

    const fn new() -> Self {
        Self {
            devices: None,
            fetched_at: None,
        }
    }

    fn get(&self) -> Option<Vec<DeviceInfo>> {
        let fetched_at = self.fetched_at?;
        if fetched_at.elapsed() > Self::TTL {
            return None;
        }
        self.devices.clone()
    }

    fn set(&mut self, devices: Vec<DeviceInfo>) {
        self.devices = Some(devices);
        self.fetched_at = Some(Instant::now());
    }

    fn invalidate(&mut self) {
        self.devices = None;
        self.fetched_at = None;
    }
}

/// Get cached device list if still valid.
pub fn get_cached_devices() -> Option<Vec<DeviceInfo>> {
    DEVICE_CACHE.lock().ok()?.get()
}

/// Store device list in cache.
pub fn cache_devices(devices: Vec<DeviceInfo>) {
    if let Ok(mut cache) = DEVICE_CACHE.lock() {
        cache.set(devices);
    }
}

/// Invalidate the device cache (call after lifecycle operations).
pub fn invalidate_cache() {
    if let Ok(mut cache) = DEVICE_CACHE.lock() {
        cache.invalidate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_mobile_core::types::TargetType;

    fn make_device(name: &str, udid: &str) -> DeviceInfo {
        DeviceInfo {
            name: name.to_string(),
            udid: udid.to_string(),
            state: Some("Booted".to_string()),
            target_type: TargetType::Simulator,
            os_version: Some("iOS 17.0".to_string()),
            architecture: None,
            companion_info: None,
        }
    }

    #[test]
    fn test_cache_miss_when_empty() {
        let cache = CacheInner::new();
        assert!(cache.get().is_none());
    }

    #[test]
    fn test_cache_hit_after_set() {
        let mut cache = CacheInner::new();
        let devices = vec![make_device("iPhone 15", "AAAA-BBBB")];
        cache.set(devices.clone());

        let cached = cache.get();
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].name, "iPhone 15");
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = CacheInner::new();
        let devices = vec![make_device("iPhone 15", "AAAA-BBBB")];
        cache.set(devices);

        cache.invalidate();
        assert!(cache.get().is_none());
    }
}
