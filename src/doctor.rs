//! `agent-mobile doctor` コマンド
//!
//! 外部依存関係の状態を診断し、環境セットアップの問題を報告します。

use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use crate::helpers::format::OutputFormat;

#[derive(Args)]
pub struct DoctorArgs {
    /// Output format
    #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Text)]
    pub format: OutputFormat,

    /// Output in JSON format (alias for --format json)
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Ok,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckCategory {
    Ios,
    Android,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub category: CheckCategory,
    pub status: CheckStatus,
    pub message: String,
    pub critical: bool,
}

#[derive(Debug, Serialize)]
pub struct DoctorSummary {
    pub ok: usize,
    pub warnings: usize,
    pub errors: usize,
}

#[derive(Debug, Serialize)]
pub struct DoctorReport {
    pub checks: Vec<CheckResult>,
    pub summary: DoctorSummary,
}

pub async fn run(mut args: DoctorArgs) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // --json フラグが指定されている場合は format を Json に設定
    if args.json {
        args.format = OutputFormat::Json;
    }

    let checks = vec![
        // iOS checks
        check_xcode(),
        check_simctl(),
        check_coresimulator(),
        check_xcuitest_runner(),
        // Android checks
        check_adb_server(),
        check_android_sdk(),
        check_android_emulator(),
    ];

    let summary = DoctorSummary {
        ok: checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Ok))
            .count(),
        warnings: checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Warning))
            .count(),
        errors: checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Error))
            .count(),
    };

    let has_critical_error = checks
        .iter()
        .any(|c| c.critical && matches!(c.status, CheckStatus::Error));

    let report = DoctorReport { checks, summary };

    if args.format.is_json() {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_text_report(&report);
    }

    if has_critical_error {
        return Err("Critical environment errors detected. See report above for details.".into());
    }

    Ok(())
}

fn print_text_report(report: &DoctorReport) {
    println!("agent-mobile doctor");
    println!("{}", "=".repeat(60));

    // Group by category
    println!("\n[iOS]");
    for check in report
        .checks
        .iter()
        .filter(|c| matches!(c.category, CheckCategory::Ios))
    {
        print_check(check);
    }

    println!("\n[Android]");
    for check in report
        .checks
        .iter()
        .filter(|c| matches!(c.category, CheckCategory::Android))
    {
        print_check(check);
    }

    println!(
        "\nSummary: {} ok, {} warning(s), {} error(s)",
        report.summary.ok, report.summary.warnings, report.summary.errors
    );
}

fn print_check(check: &CheckResult) {
    let icon = match check.status {
        CheckStatus::Ok => "\u{2713}", // ✓
        CheckStatus::Warning => "!",
        CheckStatus::Error => "\u{2717}", // ✗
    };
    println!("  {} {} - {}", icon, check.name, check.message);
}

// =============================================================================
// iOS Checks
// =============================================================================

fn check_xcode() -> CheckResult {
    let name = "Xcode".to_string();
    let category = CheckCategory::Ios;
    let critical = true;

    match std::process::Command::new("xcode-select")
        .arg("-p")
        .output()
    {
        Ok(output) if output.status.success() => {
            let dev_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
            CheckResult {
                name,
                category,
                status: CheckStatus::Ok,
                message: format!("Developer directory: {}", dev_dir),
                critical,
            }
        }
        Ok(_) => CheckResult {
            name,
            category,
            status: CheckStatus::Error,
            message: "Xcode command line tools not installed. Run: xcode-select --install"
                .to_string(),
            critical,
        },
        Err(e) => CheckResult {
            name,
            category,
            status: CheckStatus::Error,
            message: format!("xcode-select not found: {}", e),
            critical,
        },
    }
}

fn check_simctl() -> CheckResult {
    let name = "simctl".to_string();
    let category = CheckCategory::Ios;
    let critical = true;

    match std::process::Command::new("xcrun")
        .args(["simctl", "help"])
        .output()
    {
        Ok(output) if output.status.success() => CheckResult {
            name,
            category,
            status: CheckStatus::Ok,
            message: "xcrun simctl is available".to_string(),
            critical,
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            CheckResult {
                name,
                category,
                status: CheckStatus::Error,
                message: format!("xcrun simctl failed: {}", stderr),
                critical,
            }
        }
        Err(e) => CheckResult {
            name,
            category,
            status: CheckStatus::Error,
            message: format!("xcrun not found: {}", e),
            critical,
        },
    }
}

fn check_coresimulator() -> CheckResult {
    let name = "CoreSimulator".to_string();
    let category = CheckCategory::Ios;
    let critical = true;

    let framework_paths = [
        "/Applications/Xcode.app/Contents/Developer/Library/PrivateFrameworks/CoreSimulator.framework",
        "/Applications/Xcode-beta.app/Contents/Developer/Library/PrivateFrameworks/CoreSimulator.framework",
        "/Library/Developer/PrivateFrameworks/CoreSimulator.framework",
    ];

    // Check static paths first
    for path in &framework_paths {
        if PathBuf::from(path).exists() {
            return CheckResult {
                name,
                category,
                status: CheckStatus::Ok,
                message: format!("Framework found: {}", path),
                critical,
            };
        }
    }

    // Fallback: use xcode-select -p to find developer directory
    if let Ok(output) = std::process::Command::new("xcode-select")
        .arg("-p")
        .output()
    {
        if output.status.success() {
            let dev_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let path = format!(
                "{}/Library/PrivateFrameworks/CoreSimulator.framework",
                dev_dir
            );
            if PathBuf::from(&path).exists() {
                return CheckResult {
                    name,
                    category,
                    status: CheckStatus::Ok,
                    message: format!("Framework found: {}", path),
                    critical,
                };
            }
        }
    }

    CheckResult {
        name,
        category,
        status: CheckStatus::Error,
        message: "CoreSimulator.framework not found. Ensure Xcode is installed.".to_string(),
        critical,
    }
}

fn check_xcuitest_runner() -> CheckResult {
    let name = "XCUITest Runner".to_string();
    let category = CheckCategory::Ios;
    let critical = false;

    match agent_mobile_platform_ios::xcuitest::RunnerBuildProducts::find() {
        Some(products) => CheckResult {
            name,
            category,
            status: CheckStatus::Ok,
            message: format!("Build products found: {}", products.host_app.display()),
            critical,
        },
        None => CheckResult {
            name,
            category,
            status: CheckStatus::Warning,
            message: format!(
                "Build products not found (will be built on first use). Searched: {}",
                agent_mobile_platform_ios::xcuitest::RunnerBuildProducts::searched_paths_description()
            ),
            critical,
        },
    }
}

// =============================================================================
// Android Checks
// =============================================================================

fn check_adb_server() -> CheckResult {
    let name = "ADB Server".to_string();
    let category = CheckCategory::Android;
    let critical = true;

    if agent_mobile_platform_android::is_adb_available() {
        CheckResult {
            name,
            category,
            status: CheckStatus::Ok,
            message: "ADB server is reachable at 127.0.0.1:5037".to_string(),
            critical,
        }
    } else {
        CheckResult {
            name,
            category,
            status: CheckStatus::Error,
            message: "ADB server not reachable at 127.0.0.1:5037. Run: adb start-server"
                .to_string(),
            critical,
        }
    }
}

fn check_android_sdk() -> CheckResult {
    let name = "Android SDK".to_string();
    let category = CheckCategory::Android;
    let critical = false;

    // Check ANDROID_HOME
    if let Ok(home) = std::env::var("ANDROID_HOME") {
        if PathBuf::from(&home).exists() {
            return CheckResult {
                name,
                category,
                status: CheckStatus::Ok,
                message: format!("ANDROID_HOME: {}", home),
                critical,
            };
        } else {
            return CheckResult {
                name,
                category,
                status: CheckStatus::Warning,
                message: format!("ANDROID_HOME set but path does not exist: {}", home),
                critical,
            };
        }
    }

    // Check ANDROID_SDK_ROOT
    if let Ok(root) = std::env::var("ANDROID_SDK_ROOT") {
        if PathBuf::from(&root).exists() {
            return CheckResult {
                name,
                category,
                status: CheckStatus::Ok,
                message: format!("ANDROID_SDK_ROOT: {}", root),
                critical,
            };
        } else {
            return CheckResult {
                name,
                category,
                status: CheckStatus::Warning,
                message: format!("ANDROID_SDK_ROOT set but path does not exist: {}", root),
                critical,
            };
        }
    }

    CheckResult {
        name,
        category,
        status: CheckStatus::Warning,
        message: "ANDROID_HOME / ANDROID_SDK_ROOT not set".to_string(),
        critical,
    }
}

fn check_android_emulator() -> CheckResult {
    let name = "Android Emulator".to_string();
    let category = CheckCategory::Android;
    let critical = false;

    match std::process::Command::new("emulator")
        .arg("-version")
        .output()
    {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version_line = stdout.lines().next().unwrap_or("unknown version");
            CheckResult {
                name,
                category,
                status: CheckStatus::Ok,
                message: version_line.to_string(),
                critical,
            }
        }
        _ => CheckResult {
            name,
            category,
            status: CheckStatus::Warning,
            message: "emulator CLI not found on PATH".to_string(),
            critical,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_status_serialize() {
        let ok = serde_json::to_string(&CheckStatus::Ok).unwrap();
        assert_eq!(ok, r#""ok""#);
        let warning = serde_json::to_string(&CheckStatus::Warning).unwrap();
        assert_eq!(warning, r#""warning""#);
        let error = serde_json::to_string(&CheckStatus::Error).unwrap();
        assert_eq!(error, r#""error""#);
    }

    #[test]
    fn test_check_category_serialize() {
        let ios = serde_json::to_string(&CheckCategory::Ios).unwrap();
        assert_eq!(ios, r#""ios""#);
        let android = serde_json::to_string(&CheckCategory::Android).unwrap();
        assert_eq!(android, r#""android""#);
    }

    #[test]
    fn test_check_result_serialize() {
        let result = CheckResult {
            name: "Test".to_string(),
            category: CheckCategory::Ios,
            status: CheckStatus::Ok,
            message: "All good".to_string(),
            critical: true,
        };
        let json: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(json["name"], "Test");
        assert_eq!(json["category"], "ios");
        assert_eq!(json["status"], "ok");
        assert_eq!(json["message"], "All good");
        assert_eq!(json["critical"], true);
    }

    #[test]
    fn test_doctor_report_serialize() {
        let report = DoctorReport {
            checks: vec![
                CheckResult {
                    name: "Xcode".to_string(),
                    category: CheckCategory::Ios,
                    status: CheckStatus::Ok,
                    message: "Found".to_string(),
                    critical: true,
                },
                CheckResult {
                    name: "ADB".to_string(),
                    category: CheckCategory::Android,
                    status: CheckStatus::Error,
                    message: "Not found".to_string(),
                    critical: true,
                },
            ],
            summary: DoctorSummary {
                ok: 1,
                warnings: 0,
                errors: 1,
            },
        };
        let json: serde_json::Value = serde_json::to_value(&report).unwrap();
        assert_eq!(json["checks"].as_array().unwrap().len(), 2);
        assert_eq!(json["summary"]["ok"], 1);
        assert_eq!(json["summary"]["warnings"], 0);
        assert_eq!(json["summary"]["errors"], 1);
    }

    #[test]
    fn test_summary_counts() {
        let checks = [
            CheckResult {
                name: "A".to_string(),
                category: CheckCategory::Ios,
                status: CheckStatus::Ok,
                message: "ok".to_string(),
                critical: false,
            },
            CheckResult {
                name: "B".to_string(),
                category: CheckCategory::Ios,
                status: CheckStatus::Warning,
                message: "warn".to_string(),
                critical: false,
            },
            CheckResult {
                name: "C".to_string(),
                category: CheckCategory::Android,
                status: CheckStatus::Error,
                message: "err".to_string(),
                critical: true,
            },
        ];

        let summary = DoctorSummary {
            ok: checks
                .iter()
                .filter(|c| matches!(c.status, CheckStatus::Ok))
                .count(),
            warnings: checks
                .iter()
                .filter(|c| matches!(c.status, CheckStatus::Warning))
                .count(),
            errors: checks
                .iter()
                .filter(|c| matches!(c.status, CheckStatus::Error))
                .count(),
        };

        assert_eq!(summary.ok, 1);
        assert_eq!(summary.warnings, 1);
        assert_eq!(summary.errors, 1);
    }
}
