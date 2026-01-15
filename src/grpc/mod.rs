pub mod client;

pub use client::IdbClient;

// Include generated proto code
pub mod idb {
    tonic::include_proto!("idb");
}
