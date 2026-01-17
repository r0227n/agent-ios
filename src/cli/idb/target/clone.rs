//! Clone a simulator

use crate::simctl;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(udid: String) -> CommandResult {
    let new_udid = simctl::clone(&udid)?;
    println!("{}", new_udid);

    Ok(())
}
