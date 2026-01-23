//! Shutdown a simulator

use agent_mobile_platform_ios::simctl;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(udid: String) -> CommandResult {
    simctl::shutdown(&udid)?;
    println!("Shutdown {}", udid);

    Ok(())
}
