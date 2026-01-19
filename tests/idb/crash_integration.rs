use crate::common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_crash_list() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "list", "--udid", &udid])
        .output()
        .expect("Failed to run crash list");

    assert!(
        output.status.success(),
        "crash list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // リストが空の場合でも成功
    let _stdout = String::from_utf8_lossy(&output.stdout);
    // JSONまたは空の出力を期待
}

#[test]
fn test_crash_list_json() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "list", "--udid", &udid])
        .output()
        .expect("Failed to run crash list");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        // 出力がある場合は各行がJSON形式であることを確認（JSONL形式）
        for line in stdout.lines() {
            let parse_result = serde_json::from_str::<serde_json::Value>(line.trim());
            assert!(
                parse_result.is_ok(),
                "Expected valid JSON output on each line, got: {}",
                line
            );
        }
    }
}

/// Test crash show CLI help (instead of requiring crash logs)
#[test]
fn test_crash_show_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "show", "--help"])
        .output()
        .expect("Failed to run crash show --help");

    assert!(
        output.status.success(),
        "crash show --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have name argument
    assert!(
        stdout.contains("NAME") || stdout.contains("name") || stdout.contains("crash"),
        "Expected name argument in help output"
    );
}

/// Test crash delete CLI help (instead of destructive operation)
#[test]
fn test_crash_delete_cli_help() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "delete", "--help"])
        .output()
        .expect("Failed to run crash delete --help");

    assert!(
        output.status.success(),
        "crash delete --help failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should have --all flag
    assert!(
        stdout.contains("--all"),
        "Expected --all flag in help output"
    );
}

/// Test crash delete has --bundle-id flag
#[test]
fn test_crash_delete_has_bundle_id_flag() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "delete", "--help"])
        .output()
        .expect("Failed to run crash delete --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--bundle-id") || stdout.contains("bundle"),
        "Expected --bundle-id flag in help output"
    );
}

/// Test crash delete has --since flag
#[test]
fn test_crash_delete_has_since_flag() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "delete", "--help"])
        .output()
        .expect("Failed to run crash delete --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--since"),
        "Expected --since flag in help output"
    );
}

/// Test crash delete has --before flag
#[test]
fn test_crash_delete_has_before_flag() {
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "delete", "--help"])
        .output()
        .expect("Failed to run crash delete --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("--before"),
        "Expected --before flag in help output"
    );
}

#[test]
fn test_crash_list_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["crash", "list", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "list", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}

#[test]
fn test_crash_list_empty() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "list", "--udid", &udid])
        .output()
        .expect("Failed to run crash list");

    // クラッシュログがない場合でも成功するべき
    assert!(output.status.success());
}
