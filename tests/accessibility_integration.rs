mod common;

use common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_accessibility_describe_all() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "accessibility-describe-all", "--udid", &udid])
        .output()
        .expect("Failed to run accessibility-describe-all");

    assert!(
        output.status.success(),
        "accessibility-describe-all failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output is valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "Expected non-empty output from describe-all"
    );

    let parse_result = serde_json::from_str::<serde_json::Value>(stdout.trim());
    assert!(
        parse_result.is_ok(),
        "Expected valid JSON output, got: {}",
        stdout
    );
}

#[test]
fn test_accessibility_describe_all_json_structure() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "accessibility-describe-all", "--udid", &udid])
        .output()
        .expect("Failed to run accessibility-describe-all");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("Failed to parse JSON");

    // Verify JSON is an array or object
    assert!(
        json.is_array() || json.is_object(),
        "Expected array or object, got: {:?}",
        json
    );
}

#[test]
fn test_accessibility_describe_all_nested() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "accessibility-describe-all",
            "--nested",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run accessibility-describe-all");

    assert!(
        output.status.success(),
        "accessibility-describe-all --nested failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parse_result = serde_json::from_str::<serde_json::Value>(stdout.trim());
    assert!(
        parse_result.is_ok(),
        "Expected valid JSON output with --nested"
    );
}

#[test]
fn test_accessibility_describe_point() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "accessibility-describe-point",
            "100",
            "100",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run accessibility-describe-point");

    assert!(
        output.status.success(),
        "accessibility-describe-point failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify output is valid JSON
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        let parse_result = serde_json::from_str::<serde_json::Value>(stdout.trim());
        assert!(
            parse_result.is_ok(),
            "Expected valid JSON output, got: {}",
            stdout
        );
    }
}

#[test]
fn test_accessibility_describe_point_center() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 画面中央付近の座標
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "accessibility-describe-point",
            "200",
            "400",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run accessibility-describe-point");

    assert!(output.status.success());
}

#[test]
fn test_accessibility_describe_point_top_left() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 画面左上の座標
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "accessibility-describe-point",
            "0",
            "0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run accessibility-describe-point");

    assert!(output.status.success());
}

#[test]
fn test_accessibility_describe_point_nested() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "accessibility-describe-point",
            "100",
            "100",
            "--nested",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run accessibility-describe-point");

    assert!(
        output.status.success(),
        "accessibility-describe-point --nested failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        let parse_result = serde_json::from_str::<serde_json::Value>(stdout.trim());
        assert!(
            parse_result.is_ok(),
            "Expected valid JSON output with --nested"
        );
    }
}

#[test]
fn test_accessibility_describe_point_negative_coordinates() {
    let udid = get_available_udid();

    // 負の座標
    let _output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "accessibility-describe-point",
            "-10",
            "-10",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run accessibility-describe-point");

    // 負の座標はエラーになる可能性がある
    // （実装依存）
}

#[test]
fn test_accessibility_describe_all_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["accessibility-describe-all", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "accessibility-describe-all", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );

    // 両方の出力がJSONとしてパース可能であることを確認
    if python_output.status.success() {
        let python_stdout = String::from_utf8_lossy(&python_output.stdout);
        let rust_stdout = String::from_utf8_lossy(&rust_output.stdout);

        let python_json = serde_json::from_str::<serde_json::Value>(python_stdout.trim());
        let rust_json = serde_json::from_str::<serde_json::Value>(rust_stdout.trim());

        assert!(python_json.is_ok(), "Python output is not valid JSON");
        assert!(rust_json.is_ok(), "Rust output is not valid JSON");
    }
}

#[test]
fn test_accessibility_describe_point_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args([
            "accessibility-describe-point",
            "100",
            "100",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "accessibility-describe-point",
            "100",
            "100",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}
