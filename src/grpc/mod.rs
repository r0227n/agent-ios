pub mod client;

pub use client::{IdbClient, LaunchConfig};

// Include generated proto code
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
pub mod idb {
    tonic::include_proto!("idb");
}
