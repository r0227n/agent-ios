pub mod client;

pub use client::{IdbClient, LaunchConfig};

// Include generated proto code
pub mod idb {
    tonic::include_proto!("idb");
}
