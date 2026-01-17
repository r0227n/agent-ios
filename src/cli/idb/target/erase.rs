//! Erase a simulator (reset to clean state)

use crate::simctl;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(udid: String) -> CommandResult {
    simctl::erase(&udid)?;
    println!("Erased {}", udid);

    Ok(())
}
