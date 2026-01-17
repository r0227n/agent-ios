mod common;

use common::get_available_udid;
use std::process::Command;

/// Helper function to delete a simulator by UDID
fn delete_simulator(udid: &str) {
    let _ = Command::new("xcrun")
        .args(["simctl", "delete", udid])
        .output();
}

/// Helper function to get available device types
fn get_available_device_type() -> Option<String> {
    let output = Command::new("xcrun")
        .args(["simctl", "list", "devicetypes", "-j"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;

    // Get first iPhone device type
    if let Some(device_types) = json.get("devicetypes").and_then(|v| v.as_array()) {
        for device_type in device_types {
            if let Some(identifier) = device_type.get("identifier").and_then(|v| v.as_str()) {
                if identifier.contains("iPhone") {
                    return Some(identifier.to_string());
                }
            }
        }
    }

    None
}

/// Helper function to get available runtimes
fn get_available_runtime() -> Option<String> {
    let output = Command::new("xcrun")
        .args(["simctl", "list", "runtimes", "-j"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).ok()?;

    // Get first available iOS runtime
    if let Some(runtimes) = json.get("runtimes").and_then(|v| v.as_array()) {
        for runtime in runtimes {
            if let Some(is_available) = runtime.get("isAvailable").and_then(|v| v.as_bool()) {
                if is_available {
                    if let Some(identifier) = runtime.get("identifier").and_then(|v| v.as_str()) {
                        if identifier.contains("iOS") {
                            return Some(identifier.to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_target_create() {
    let device_type = match get_available_device_type() {
        Some(dt) => dt,
        None => {
            eprintln!("No device type available, skipping test");
            return;
        }
    };

    let runtime = match get_available_runtime() {
        Some(r) => r,
        None => {
            eprintln!("No runtime available, skipping test");
            return;
        }
    };

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "target",
            "create",
            "TestSimulator",
            &device_type,
            &runtime,
        ])
        .output()
        .expect("Failed to run target create");

    assert!(
        output.status.success(),
        "target create failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    // 作成されたシミュレータのUDIDが出力される
    let created_udid = stdout.trim();

    // クリーンアップ: 作成したシミュレータを削除
    if !created_udid.is_empty() {
        delete_simulator(created_udid);
    }
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_target_clone() {
    let source_udid = get_available_udid();

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "clone", &source_udid, "ClonedSimulator"])
        .output()
        .expect("Failed to run target clone");

    assert!(
        output.status.success(),
        "target clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cloned_udid = stdout.trim();

    // クリーンアップ: クローンしたシミュレータを削除
    if !cloned_udid.is_empty() {
        delete_simulator(cloned_udid);
    }
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_target_delete() {
    // まずテスト用のシミュレータを作成
    let device_type = match get_available_device_type() {
        Some(dt) => dt,
        None => {
            eprintln!("No device type available, skipping test");
            return;
        }
    };

    let runtime = match get_available_runtime() {
        Some(r) => r,
        None => {
            eprintln!("No runtime available, skipping test");
            return;
        }
    };

    let create_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "target",
            "create",
            "ToBeDeleted",
            &device_type,
            &runtime,
        ])
        .output()
        .expect("Failed to create test simulator");

    if !create_output.status.success() {
        eprintln!("Failed to create test simulator, skipping delete test");
        return;
    }

    let created_udid = String::from_utf8_lossy(&create_output.stdout)
        .trim()
        .to_string();

    // 削除実行
    let delete_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "delete", &created_udid])
        .output()
        .expect("Failed to run target delete");

    assert!(
        delete_output.status.success(),
        "target delete failed: {}",
        String::from_utf8_lossy(&delete_output.stderr)
    );
}

#[test]
#[ignore] // 非常に危険な操作のため、手動実行のみ
fn test_target_delete_all() {
    // このテストは実行しないことを推奨
    // すべてのシミュレータが削除されてしまう
    eprintln!("WARNING: This test will delete ALL simulators!");
    eprintln!("Skipping for safety. Run manually if needed.");
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_target_create_python_compatibility() {
    let device_type = match get_available_device_type() {
        Some(dt) => dt,
        None => {
            eprintln!("No device type available, skipping test");
            return;
        }
    };

    let runtime = match get_available_runtime() {
        Some(r) => r,
        None => {
            eprintln!("No runtime available, skipping test");
            return;
        }
    };

    // Python idb
    let python_output = Command::new("idb")
        .args(["target", "create", "PythonTestSim", &device_type, &runtime])
        .output()
        .expect("Failed to execute Python idb");

    let python_udid = String::from_utf8_lossy(&python_output.stdout)
        .trim()
        .to_string();

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "target",
            "create",
            "RustTestSim",
            &device_type,
            &runtime,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    let rust_udid = String::from_utf8_lossy(&rust_output.stdout)
        .trim()
        .to_string();

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );

    // クリーンアップ
    if !python_udid.is_empty() {
        delete_simulator(&python_udid);
    }
    if !rust_udid.is_empty() {
        delete_simulator(&rust_udid);
    }
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_target_clone_python_compatibility() {
    let source_udid = get_available_udid();

    // Python idb
    let python_output = Command::new("idb")
        .args(["target", "clone", &source_udid, "PythonClone"])
        .output()
        .expect("Failed to execute Python idb");

    let python_udid = String::from_utf8_lossy(&python_output.stdout)
        .trim()
        .to_string();

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "clone", &source_udid, "RustClone"])
        .output()
        .expect("Failed to run agent-mobile");

    let rust_udid = String::from_utf8_lossy(&rust_output.stdout)
        .trim()
        .to_string();

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );

    // クリーンアップ
    if !python_udid.is_empty() {
        delete_simulator(&python_udid);
    }
    if !rust_udid.is_empty() {
        delete_simulator(&rust_udid);
    }
}

#[test]
#[ignore] // 破壊的操作のため、デフォルトでは実行しない
fn test_target_delete_python_compatibility() {
    let device_type = match get_available_device_type() {
        Some(dt) => dt,
        None => {
            eprintln!("No device type available, skipping test");
            return;
        }
    };

    let runtime = match get_available_runtime() {
        Some(r) => r,
        None => {
            eprintln!("No runtime available, skipping test");
            return;
        }
    };

    // テスト用シミュレータを2つ作成
    let create_output1 = Command::new("idb")
        .args(["target", "create", "DeleteTest1", &device_type, &runtime])
        .output()
        .expect("Failed to create test simulator");

    let udid1 = String::from_utf8_lossy(&create_output1.stdout)
        .trim()
        .to_string();

    let create_output2 = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "target",
            "create",
            "DeleteTest2",
            &device_type,
            &runtime,
        ])
        .output()
        .expect("Failed to create test simulator");

    let udid2 = String::from_utf8_lossy(&create_output2.stdout)
        .trim()
        .to_string();

    // Python idbで削除
    let python_output = Command::new("idb")
        .args(["target", "delete", &udid1])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobileで削除
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "target", "delete", &udid2])
        .output()
        .expect("Failed to run agent-mobile");

    // 両方とも成功するはず
    assert_eq!(
        python_output.status.success(),
        rust_output.status.success(),
        "Exit codes differ"
    );
}
