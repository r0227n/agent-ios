//! Intelligent simulator/emulator suggestion for AI agents.
//!
//! This module provides suggestions for the best available simulators/emulators
//! based on platform-specific criteria like model popularity, OS version, and state.
//!
//! Supports progressive disclosure with concise summaries by default.

use crate::platform::ios::SimulatorLister;
use crate::platform::{DeviceSuggester, DeviceSuggestion};

/// Run the suggest-simulator command.
pub async fn run(
    platform: &str,
    count: usize,
    json_output: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Select platform-specific implementation
    let suggester: Box<dyn DeviceSuggester> = match platform {
        "ios" => Box::new(SimulatorLister::new()?),
        "android" => {
            eprintln!("Android support is not yet implemented");
            return Ok(());
        }
        _ => {
            eprintln!("Invalid platform: {}. Use 'ios' or 'android'", platform);
            std::process::exit(1);
        }
    };

    // Get suggestions via trait
    let suggestions = suggester.suggest(count).await?;

    // Output
    if json_output {
        println!("{}", serde_json::to_string_pretty(&suggestions)?);
    } else {
        print_human_readable(&suggestions, suggester.platform_name());
    }

    Ok(())
}

/// Print suggestions in human-readable format.
fn print_human_readable(suggestions: &[DeviceSuggestion], platform: &str) {
    if suggestions.is_empty() {
        println!("No {} simulators available", platform);
        return;
    }

    println!("Available {} Simulators:\n", platform.to_uppercase());

    for (i, suggestion) in suggestions.iter().enumerate() {
        let os_info = suggestion
            .os_version
            .as_ref()
            .map(|v| format!(" ({})", v))
            .unwrap_or_default();

        println!("{}. {}{}", i + 1, suggestion.name, os_info);

        if !suggestion.reasons.is_empty() {
            println!("   {}", suggestion.reasons.join(", "));
        }

        println!("   UDID: {}", suggestion.udid);

        if let Some(ref state) = suggestion.state {
            println!("   State: {}", state);
        }

        println!();
    }
}
