//! Delete a simulator

use crate::simctl;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(udid: Option<String>, delete_all: bool) -> CommandResult {
    if delete_all {
        simctl::delete_all()?;
        println!("Deleted all simulators");
    } else if let Some(udid) = udid {
        simctl::delete(&udid)?;
        println!("Deleted {}", udid);
    } else {
        return Err("Either udid or --all must be specified".into());
    }

    Ok(())
}
