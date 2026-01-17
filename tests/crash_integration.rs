mod common;

use common::{ensure_companion_running, get_available_udid};
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
    let stdout = String::from_utf8_lossy(&output.stdout);
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
        // 出力がある場合はJSON形式であることを確認
        let parse_result = serde_json::from_str::<serde_json::Value>(stdout.trim());
        assert!(
            parse_result.is_ok(),
            "Expected valid JSON output, got: {}",
            stdout
        );
    }
}

#[test]
#[ignore] // クラッシュログがある環境でのみ実行可能
fn test_crash_show() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // まずクラッシュリストを取得
    let list_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "list", "--udid", &udid])
        .output()
        .expect("Failed to run crash list");

    assert!(list_output.status.success());

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    if !stdout.trim().is_empty() {
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("Failed to parse crash list JSON");

        // クラッシュログがある場合、最初のものを表示
        if let Some(crashes) = json.as_array() {
            if let Some(first_crash) = crashes.first() {
                if let Some(crash_name) = first_crash.get("name").and_then(|v| v.as_str()) {
                    let show_output = Command::new("./target/debug/agent-mobile")
                        .args(["idb", "crash", "show", crash_name, "--udid", &udid])
                        .output()
                        .expect("Failed to run crash show");

                    assert!(
                        show_output.status.success(),
                        "crash show failed: {}",
                        String::from_utf8_lossy(&show_output.stderr)
                    );

                    let show_stdout = String::from_utf8_lossy(&show_output.stdout);
                    assert!(!show_stdout.trim().is_empty(), "Expected crash log content");
                }
            }
        }
    }
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_crash_delete_all() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "delete", "--all", "--udid", &udid])
        .output()
        .expect("Failed to run crash delete");

    assert!(
        output.status.success(),
        "crash delete --all failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_crash_delete_by_bundle_id() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "crash",
            "delete",
            "--bundle-id",
            "com.apple.Preferences",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run crash delete");

    assert!(output.status.success());
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_crash_delete_since() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 1時間前以降のクラッシュログを削除
    let since_timestamp = chrono::Utc::now().timestamp() - 3600;

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "crash",
            "delete",
            "--since",
            &since_timestamp.to_string(),
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run crash delete");

    assert!(output.status.success());
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_crash_delete_before() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 1日前以前のクラッシュログを削除
    let before_timestamp = chrono::Utc::now().timestamp() - 86400;

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "crash",
            "delete",
            "--before",
            &before_timestamp.to_string(),
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run crash delete");

    assert!(output.status.success());
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_crash_delete_specific_crash() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // まずクラッシュリストを取得
    let list_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "crash", "list", "--udid", &udid])
        .output()
        .expect("Failed to run crash list");

    let stdout = String::from_utf8_lossy(&list_output.stdout);
    if !stdout.trim().is_empty() {
        let json: serde_json::Value =
            serde_json::from_str(stdout.trim()).expect("Failed to parse crash list JSON");

        if let Some(crashes) = json.as_array() {
            if let Some(first_crash) = crashes.first() {
                if let Some(crash_name) = first_crash.get("name").and_then(|v| v.as_str()) {
                    let delete_output = Command::new("./target/debug/agent-mobile")
                        .args(["idb", "crash", "delete", crash_name, "--udid", &udid])
                        .output()
                        .expect("Failed to run crash delete");

                    assert!(delete_output.status.success());
                }
            }
        }
    }
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
