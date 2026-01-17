//! Shutdown a simulator

use crate::simctl;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(udid: String) -> CommandResult {
    simctl::shutdown(&udid)?;
    println!("Shutdown {}", udid);

    Ok(())
}
