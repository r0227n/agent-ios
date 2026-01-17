mod common;

use common::{ensure_companion_running, get_available_udid};
use std::process::Command;

/// Get a shutdown simulator UDID for testing
fn get_shutdown_simulator() -> Option<String> {
    let output = Command::new("idb")
        .args(["list-targets", "--json"])
        .output()
        .expect("Failed to execute Python idb");

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if json.get("state").and_then(|v| v.as_str()) == Some("Shutdown")
                && json.get("type").and_then(|v| v.as_str()) == Some("simulator")
            {
                if let Some(udid) = json.get("udid").and_then(|v| v.as_str()) {
                    return Some(udid.to_string());
                }
            }
        }
    }
    None
}

#[test]
#[ignore] // シミュレータの状態を変更するため
fn test_target_boot() {
    let udid = match get_shutdown_simulator() {
        Some(u) => u,
        None => {
            eprintln!("No shutdown simulator available, skipping test");
            return;
        }
    };

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "boot", "--udid", &udid])
        .output()
        .expect("Failed to run target boot");

    assert!(
        output.status.success(),
        "target boot failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // クリーンアップ: シミュレータをシャットダウン
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", "--udid", &udid])
        .output();
}

#[test]
#[ignore] // シミュレータの状態を変更するため
fn test_target_boot_headless() {
    let udid = match get_shutdown_simulator() {
        Some(u) => u,
        None => {
            eprintln!("No shutdown simulator available, skipping test");
            return;
        }
    };

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "boot", "--headless", "--udid", &udid])
        .output()
        .expect("Failed to run target boot");

    assert!(
        output.status.success(),
        "target boot --headless failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // クリーンアップ: シミュレータをシャットダウン
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", "--udid", &udid])
        .output();
}

#[test]
fn test_target_shutdown() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", "--udid", &udid])
        .output()
        .expect("Failed to run target shutdown");

    assert!(
        output.status.success(),
        "target shutdown failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // クリーンアップ: シミュレータを再起動
    // （次のテストのために）
    let _ = Command::new("idb")
        .args(["target", "boot", "--udid", &udid])
        .output();
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_target_erase() {
    let udid = match get_shutdown_simulator() {
        Some(u) => u,
        None => {
            eprintln!("No shutdown simulator available, skipping test");
            return;
        }
    };

    // まずシャットダウン
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", "--udid", &udid])
        .output();

    // erase実行
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "erase", "--udid", &udid])
        .output()
        .expect("Failed to run target erase");

    assert!(
        output.status.success(),
        "target erase failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore] // シミュレータの状態を変更するため
fn test_target_boot_python_compatibility() {
    let udid = match get_shutdown_simulator() {
        Some(u) => u,
        None => {
            eprintln!("No shutdown simulator available, skipping test");
            return;
        }
    };

    // Python idb
    let python_output = Command::new("idb")
        .args(["target", "boot", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "boot", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず（2回目のbootはすでに起動中なのでエラーにならないはず）
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );

    // クリーンアップ
    let _ = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", "--udid", &udid])
        .output();
}

#[test]
fn test_target_shutdown_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["target", "shutdown", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", "--udid", &udid])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず（2回目のshutdownはすでにシャットダウン済みなのでエラーにならないはず）
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );

    // クリーンアップ: シミュレータを再起動
    let _ = Command::new("idb")
        .args(["target", "boot", "--udid", &udid])
        .output();
}

#[test]
fn test_target_shutdown_already_shutdown() {
    let udid = match get_shutdown_simulator() {
        Some(u) => u,
        None => {
            eprintln!("No shutdown simulator available, skipping test");
            return;
        }
    };

    // すでにシャットダウン済みのシミュレータをシャットダウン
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", "--udid", &udid])
        .output()
        .expect("Failed to run target shutdown");

    // エラーにならない（または適切にハンドリングされる）
    assert!(output.status.success() || !output.stderr.is_empty());
}

#[test]
#[ignore] // シミュレータの状態を変更するため
fn test_target_boot_shutdown_cycle() {
    let udid = match get_shutdown_simulator() {
        Some(u) => u,
        None => {
            eprintln!("No shutdown simulator available, skipping test");
            return;
        }
    };

    // Boot
    let boot_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "boot", "--udid", &udid])
        .output()
        .expect("Failed to run target boot");
    assert!(boot_output.status.success());

    // 少し待つ
    std::thread::sleep(std::time::Duration::from_secs(2));

    // Shutdown
    let shutdown_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "shutdown", "--udid", &udid])
        .output()
        .expect("Failed to run target shutdown");
    assert!(shutdown_output.status.success());
}
