//! Create a new simulator

use agent_mobile_platform_ios::simctl;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(name: String, device_type: String, os_version: String) -> CommandResult {
    let udid = simctl::create(&name, &device_type, &os_version)?;
    println!("{}", udid);

    Ok(())
}
