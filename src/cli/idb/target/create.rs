//! Create a new simulator

use crate::simctl;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(device_type: String, os_version: String) -> CommandResult {
    let udid = simctl::create(&device_type, &os_version)?;
    println!("{}", udid);

    Ok(())
}
