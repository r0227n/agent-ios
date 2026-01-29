//! Doctor command implementation.
//!
//! Checks the development environment for required dependencies.

mod android;
mod ios;
mod types;

use clap::Args;

use crate::helpers::client::CommandResult;
use crate::helpers::common_args::FormatArgs;
use crate::helpers::format::OutputFormat;

pub use types::{CheckResult, DoctorResults, Summary};

/// Platform filter for doctor command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum DoctorPlatform {
    /// iOS platform only.
    Ios,
    /// Android platform only.
    Android,
}

/// Doctor command arguments.
#[derive(Args, Debug)]
pub struct DoctorArgs {
    /// Check only specified platform.
    #[arg(long, value_enum)]
    pub platform: Option<DoctorPlatform>,

    /// Output format options.
    #[command(flatten)]
    pub format: FormatArgs,
}

/// Run the doctor command.
pub async fn run(args: DoctorArgs) -> CommandResult {
    let check_ios = args.platform.is_none() || args.platform == Some(DoctorPlatform::Ios);
    let check_android = args.platform.is_none() || args.platform == Some(DoctorPlatform::Android);

    let ios_results = if check_ios {
        Some(ios::check_all())
    } else {
        None
    };

    let android_results = if check_android {
        Some(android::check_all())
    } else {
        None
    };

    // Calculate summary
    let passed = ios_results.as_ref().map_or(0, |r| r.passed_count())
        + android_results.as_ref().map_or(0, |r| r.passed_count());
    let failed = ios_results.as_ref().map_or(0, |r| r.failed_count())
        + android_results.as_ref().map_or(0, |r| r.failed_count());

    let results = DoctorResults {
        ios: ios_results,
        android: android_results,
        summary: Summary { passed, failed },
    };

    // Output results
    match args.format.format {
        OutputFormat::Json => print_json(&results)?,
        OutputFormat::Text => print_text(&results),
    }

    // Return error if any check failed
    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

/// Print results as JSON.
fn print_json(results: &DoctorResults) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("{}", serde_json::to_string_pretty(results)?);
    Ok(())
}

/// Print results as human-readable text.
fn print_text(results: &DoctorResults) {
    if let Some(ref ios) = results.ios {
        println!("iOS:");
        for check in &ios.checks {
            print_check(check);
        }
        println!();
    }

    if let Some(ref android) = results.android {
        println!("Android:");
        for check in &android.checks {
            print_check(check);
        }
        println!();
    }

    // Print summary
    println!(
        "Result: {} passed, {} failed",
        results.summary.passed, results.summary.failed
    );
}

/// Print a single check result.
fn print_check(check: &CheckResult) {
    let status_icon = if check.status.is_ok() { "[OK]" } else { "[NG]" };

    // Build the description
    let mut desc = check.name.clone();
    if let Some(ref version) = check.version {
        desc.push_str(&format!(" ({})", version));
    }
    if let Some(ref path) = check.path {
        desc.push_str(&format!(" ({})", path));
    }
    if let Some(ref message) = check.message {
        desc.push_str(&format!(" {}", message));
    }

    println!("  {} {}", status_icon, desc);

    // Print hint if available
    if let Some(ref hint) = check.hint {
        println!("       -> {}", hint);
    }
}
