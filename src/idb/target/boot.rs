//! Boot a simulator

use agent_mobile_platform_ios::simctl;

pub type CommandResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

pub async fn run(udid: String, headless: bool) -> CommandResult {
    if headless {
        // Set headless environment variable before booting
        std::env::set_var("SIMCTL_CHILD_HEADLESS", "1");
    }

    simctl::boot(&udid)?;
    println!("Booted {}", udid);

    Ok(())
}
