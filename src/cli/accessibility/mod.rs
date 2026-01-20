//! Accessibility audit command implementation.
//!
//! Provides accessibility auditing capabilities for mobile apps.

use clap::Args;
use serde::Serialize;

use crate::cli::helpers::{CommandResult, OutputFormat};
use crate::types::Platform;

/// Accessibility command arguments.
#[derive(Args, Debug)]
pub struct AccessibilityArgs {
    /// Run accessibility audit on current screen.
    #[arg(long)]
    pub audit: bool,

    /// Platform (ios or android). Auto-detected if not specified.
    #[arg(short = 'p', long)]
    pub platform: Option<String>,

    /// Device UDID/serial. Auto-detected if not specified.
    #[arg(short, long)]
    pub udid: Option<String>,

    /// Output format (human or json).
    #[arg(short = 'o', long, value_enum, default_value = "human")]
    pub output: OutputFormat,
}

/// Accessibility issue found during audit.
#[derive(Debug, Serialize)]
pub struct AccessibilityIssue {
    pub severity: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Audit result.
#[derive(Debug, Serialize)]
pub struct AuditResult {
    pub total_elements: usize,
    pub issues: Vec<AccessibilityIssue>,
    pub score: f64,
}

/// Detect platform based on available devices.
async fn detect_platform() -> Result<Platform, Box<dyn std::error::Error + Send + Sync>> {
    let ios_state_path = std::path::Path::new("/tmp/idb/state");
    if ios_state_path.exists() {
        return Ok(Platform::Ios);
    }

    if crate::platform::android::adb::is_adb_available() {
        let devices = crate::platform::android::adb::list_devices();
        if let Ok(devs) = devices {
            if !devs.is_empty() {
                return Ok(Platform::Android);
            }
        }
    }

    Ok(Platform::Ios)
}

/// Execute the accessibility command.
pub async fn run(args: AccessibilityArgs) -> CommandResult {
    let platform = match &args.platform {
        Some(p) => p
            .parse::<Platform>()
            .map_err(|e: String| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?,
        None => detect_platform().await?,
    };

    if args.audit {
        return execute_audit(platform, &args).await;
    }

    // Default to audit
    execute_audit(platform, &args).await
}

/// Execute accessibility audit.
async fn execute_audit(platform: Platform, args: &AccessibilityArgs) -> CommandResult {
    match platform {
        Platform::Ios => {
            use crate::cli::helpers::with_client;

            with_client(args.udid.as_deref(), |mut client| async move {
                let json_str = client.accessibility_info(None, true).await?;
                let json: serde_json::Value = serde_json::from_str(&json_str)?;

                let (total, issues) = analyze_ios_accessibility(&json);
                let score = calculate_score(total, &issues);

                let result = AuditResult {
                    total_elements: total,
                    issues,
                    score,
                };

                if args.output.is_json() {
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    print_audit_result(&result);
                }
                Ok(())
            })
            .await
        }
        Platform::Android => {
            use crate::platform::android::adb::uiautomator;

            let xml = uiautomator::dump_ui(args.udid.as_deref()).await?;
            let elements = uiautomator::parse_ui_hierarchy(&xml)?;

            let (total, issues) = analyze_android_accessibility(&elements);
            let score = calculate_score(total, &issues);

            let result = AuditResult {
                total_elements: total,
                issues,
                score,
            };

            if args.output.is_json() {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                print_audit_result(&result);
            }
            Ok(())
        }
    }
}

/// Analyze iOS accessibility tree for issues.
fn analyze_ios_accessibility(json: &serde_json::Value) -> (usize, Vec<AccessibilityIssue>) {
    let mut total = 0;
    let mut issues = Vec::new();

    fn traverse(node: &serde_json::Value, total: &mut usize, issues: &mut Vec<AccessibilityIssue>) {
        if let Some(obj) = node.as_object() {
            *total += 1;

            let label = obj.get("AXLabel").and_then(|v| v.as_str()).unwrap_or("");
            let role = obj.get("AXRole").and_then(|v| v.as_str()).unwrap_or("");
            let value = obj.get("AXValue").and_then(|v| v.as_str()).unwrap_or("");

            // Check for missing accessibility labels on interactive elements
            let is_interactive = matches!(
                role,
                "AXButton" | "AXLink" | "AXTextField" | "AXImage" | "AXCell"
            );

            if is_interactive && label.is_empty() && value.is_empty() {
                issues.push(AccessibilityIssue {
                    severity: "warning".to_string(),
                    description: format!(
                        "Interactive element ({}) missing accessibility label",
                        role
                    ),
                    element: Some(role.to_string()),
                    suggestion: Some(
                        "Add an accessibility label to describe this element".to_string(),
                    ),
                });
            }

            // Check for image without description
            if role == "AXImage" && label.is_empty() {
                issues.push(AccessibilityIssue {
                    severity: "warning".to_string(),
                    description: "Image missing accessibility description".to_string(),
                    element: Some("AXImage".to_string()),
                    suggestion: Some(
                        "Add an accessibility label or mark as decorative".to_string(),
                    ),
                });
            }

            if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
                for child in children {
                    traverse(child, total, issues);
                }
            }
        }
    }

    traverse(json, &mut total, &mut issues);
    (total, issues)
}

/// Analyze Android UI hierarchy for accessibility issues.
fn analyze_android_accessibility(
    elements: &[crate::platform::android::adb::uiautomator::AccessibilityElement],
) -> (usize, Vec<AccessibilityIssue>) {
    let total = elements.len();
    let mut issues = Vec::new();

    for e in elements {
        let label = e.label().unwrap_or("");
        let element_type = e.element_type().unwrap_or("");

        // Check for clickable elements without labels
        if e.clickable && label.is_empty() {
            issues.push(AccessibilityIssue {
                severity: "warning".to_string(),
                description: format!(
                    "Clickable element ({}) missing accessibility label",
                    element_type
                ),
                element: e.resource_id.clone(),
                suggestion: Some("Add contentDescription attribute".to_string()),
            });
        }

        // Check for images without descriptions
        if element_type.contains("Image") && label.is_empty() {
            issues.push(AccessibilityIssue {
                severity: "warning".to_string(),
                description: "Image missing content description".to_string(),
                element: e.resource_id.clone(),
                suggestion: Some(
                    "Add contentDescription or mark as importantForAccessibility=no".to_string(),
                ),
            });
        }

        // Check for small touch targets
        if e.clickable {
            if let Some([l, t, r, b]) = e.bounds {
                let width = r - l;
                let height = b - t;
                // Android recommends minimum 48dp (roughly 48px at 1x density)
                if width < 48 || height < 48 {
                    issues.push(AccessibilityIssue {
                        severity: "info".to_string(),
                        description: format!(
                            "Touch target may be too small ({}x{} px)",
                            width, height
                        ),
                        element: e.resource_id.clone(),
                        suggestion: Some(
                            "Consider making touch target at least 48x48 dp".to_string(),
                        ),
                    });
                }
            }
        }
    }

    (total, issues)
}

/// Calculate accessibility score (0-100).
fn calculate_score(total: usize, issues: &[AccessibilityIssue]) -> f64 {
    if total == 0 {
        return 100.0;
    }

    let error_count = issues.iter().filter(|i| i.severity == "error").count();
    let warning_count = issues.iter().filter(|i| i.severity == "warning").count();

    // Weight: errors are more severe than warnings
    let penalty = (error_count * 10 + warning_count * 5) as f64;
    let max_penalty = (total * 10) as f64;

    let score = 100.0 - (penalty / max_penalty * 100.0);
    score.max(0.0).min(100.0)
}

/// Print audit result in human-readable format.
fn print_audit_result(result: &AuditResult) {
    println!("Accessibility Audit Results");
    println!("{:=<50}", "");
    println!("Total elements analyzed: {}", result.total_elements);
    println!("Issues found: {}", result.issues.len());
    println!("Score: {:.1}/100", result.score);
    println!();

    if result.issues.is_empty() {
        println!("No accessibility issues found!");
        return;
    }

    println!("Issues:");
    println!("{:-<50}", "");
    for (i, issue) in result.issues.iter().enumerate() {
        let severity_icon = match issue.severity.as_str() {
            "error" => "[!]",
            "warning" => "[?]",
            _ => "[i]",
        };
        println!("{}. {} {}", i + 1, severity_icon, issue.description);
        if let Some(elem) = &issue.element {
            println!("   Element: {}", elem);
        }
        if let Some(suggestion) = &issue.suggestion {
            println!("   Suggestion: {}", suggestion);
        }
    }
}
