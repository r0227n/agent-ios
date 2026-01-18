mod common;

use common::{ensure_companion_running, get_available_udid};
use std::process::Command;

#[test]
fn test_set_location_tokyo() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "35.6762",
            "139.6503",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    assert!(
        output.status.success(),
        "set-location Tokyo failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_set_location_new_york() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "40.7128",
            "-74.0060",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    assert!(output.status.success());
}

#[test]
fn test_set_location_london() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "51.5074",
            "-0.1278",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    assert!(output.status.success());
}

#[test]
fn test_set_location_equator() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 赤道上の点（ガボン、リーブルヴィル付近）
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "0.0",
            "9.4538",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    assert!(output.status.success());
}

#[test]
fn test_set_location_north_pole() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 北極点
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "90.0",
            "0.0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    assert!(output.status.success());
}

#[test]
fn test_set_location_south_pole() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 南極点
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "-90.0",
            "0.0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    assert!(output.status.success());
}

#[test]
fn test_set_location_180_longitude() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 経度180度（国際日付変更線）
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "0.0",
            "180.0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    assert!(output.status.success());
}

#[test]
fn test_set_location_negative_180_longitude() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // 経度-180度
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "0.0",
            "-180.0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    assert!(output.status.success());
}

#[test]
fn test_set_location_invalid_latitude_too_high() {
    let udid = get_available_udid();

    // 緯度が範囲外（>90）
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "91.0",
            "0.0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    // エラーになるべき
    assert!(!output.status.success());
}

#[test]
fn test_set_location_invalid_latitude_too_low() {
    let udid = get_available_udid();

    // 緯度が範囲外（<-90）
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "-91.0",
            "0.0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    // エラーになるべき
    assert!(!output.status.success());
}

#[test]
fn test_set_location_invalid_longitude_too_high() {
    let udid = get_available_udid();

    // 経度が範囲外（>180）
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "0.0",
            "181.0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    // エラーになるべき
    assert!(!output.status.success());
}

#[test]
fn test_set_location_invalid_longitude_too_low() {
    let udid = get_available_udid();

    // 経度が範囲外（<-180）
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "0.0",
            "-181.0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    // エラーになるべき
    assert!(!output.status.success());
}

#[test]
fn test_set_location_invalid_not_number() {
    let udid = get_available_udid();

    // 数値でない
    let output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "invalid",
            "0.0",
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run set-location");

    // エラーになるべき
    assert!(!output.status.success());
}

#[test]
fn test_set_location_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_output = Command::new("idb")
        .args(["set-location", "35.6762", "139.6503", "--udid", &udid])
        .output()
        .expect("Failed to execute Python idb");

    // agent-mobile
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "location",
            "set-location",
            "35.6762",
            "139.6503",
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
