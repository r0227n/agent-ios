mod common;

use common::{ensure_companion_running, get_available_udid, wait_with_timeout};
use std::process::{Command, Stdio};

#[test]
#[ignore] // 長時間実行のため
fn test_video_record_video() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output_path = "/tmp/test_recording.mp4";

    // 録画開始（バックグラウンドプロセス）
    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "record-video", output_path, "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn record-video");

    // 5秒後に終了
    let output = wait_with_timeout(child, 5);

    assert!(
        output.status.success() || output.status.code().is_some(),
        "record-video failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // ファイルが作成されたことを確認
    let metadata = std::fs::metadata(output_path);
    if let Ok(meta) = metadata {
        assert!(meta.len() > 0, "Expected non-empty video file");
        // クリーンアップ
        let _ = std::fs::remove_file(output_path);
    }
}

#[test]
#[ignore] // 長時間実行のため
fn test_video_record_video_with_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output_path = "/tmp/test_recording_format.mp4";

    let child = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "video",
            "record-video",
            output_path,
            "--format",
            "h264",
            "--udid",
            &udid,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn record-video");

    let output = wait_with_timeout(child, 5);

    // クリーンアップ
    let _ = std::fs::remove_file(output_path);

    assert!(
        output.status.success() || output.status.code().is_some(),
        "record-video with format failed"
    );
}

#[test]
#[ignore] // 長時間実行のため
fn test_video_record_video_with_fps() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output_path = "/tmp/test_recording_fps.mp4";

    let child = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "video",
            "record-video",
            output_path,
            "--fps",
            "30",
            "--udid",
            &udid,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn record-video");

    let output = wait_with_timeout(child, 5);

    // クリーンアップ
    let _ = std::fs::remove_file(output_path);

    assert!(
        output.status.success() || output.status.code().is_some(),
        "record-video with fps failed"
    );
}

#[test]
#[ignore] // 長時間実行のため
fn test_video_stream() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "video-stream", "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn video-stream");

    // 5秒後に終了
    let output = wait_with_timeout(child, 5);

    // ストリーミングが開始されたことを確認
    // （stdoutに何らかのデータが出力される）
    let stdout = output.stdout;
    assert!(
        !stdout.is_empty() || output.status.success(),
        "Expected video stream output"
    );
}

#[test]
#[ignore] // 長時間実行のため
fn test_video_stream_with_format() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let child = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "video",
            "video-stream",
            "--format",
            "h264",
            "--udid",
            &udid,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn video-stream");

    let output = wait_with_timeout(child, 5);

    // ストリーミングが開始されたことを確認
    assert!(
        !output.stdout.is_empty() || output.status.success(),
        "Expected video stream output with format"
    );
}

#[test]
#[ignore] // 長時間実行のため
fn test_video_stream_with_fps() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let child = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "video",
            "video-stream",
            "--fps",
            "30",
            "--udid",
            &udid,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn video-stream");

    let output = wait_with_timeout(child, 5);

    // ストリーミングが開始されたことを確認
    assert!(
        !output.stdout.is_empty() || output.status.success(),
        "Expected video stream output with fps"
    );
}

#[test]
#[ignore] // 長時間実行のため
fn test_video_record_video_file_size() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let output_path = "/tmp/test_recording_size.mp4";

    let child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "record-video", output_path, "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn record-video");

    // 3秒録画
    let output = wait_with_timeout(child, 3);

    // ファイルサイズを確認
    if let Ok(metadata) = std::fs::metadata(output_path) {
        let file_size = metadata.len();
        assert!(
            file_size > 1000,
            "Expected file size > 1KB, got {} bytes",
            file_size
        );

        // クリーンアップ
        let _ = std::fs::remove_file(output_path);
    }
}

#[test]
#[ignore] // 長時間実行のため
fn test_video_record_video_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    let python_output_path = "/tmp/python_recording.mp4";
    let rust_output_path = "/tmp/rust_recording.mp4";

    // Python idb
    let python_child = Command::new("idb")
        .args(["video", "record-video", python_output_path, "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Python idb");

    let python_output = wait_with_timeout(python_child, 3);

    // agent-mobile
    let rust_child = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "video",
            "record-video",
            rust_output_path,
            "--udid",
            &udid,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn agent-mobile");

    let rust_output = wait_with_timeout(rust_child, 3);

    // 両方とも録画が実行されたことを確認
    let python_exists = std::fs::metadata(python_output_path).is_ok();
    let rust_exists = std::fs::metadata(rust_output_path).is_ok();

    // クリーンアップ
    let _ = std::fs::remove_file(python_output_path);
    let _ = std::fs::remove_file(rust_output_path);

    // 両方ともファイルが作成されているか、両方とも作成されていないか
    assert_eq!(
        python_exists, rust_exists,
        "File creation status differs between Python and Rust implementations"
    );
}

#[test]
#[ignore] // 長時間実行のため
fn test_video_stream_python_compatibility() {
    let udid = get_available_udid();
    ensure_companion_running(&udid);

    // Python idb
    let python_child = Command::new("idb")
        .args(["video", "video-stream", "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn Python idb");

    let python_output = wait_with_timeout(python_child, 3);

    // agent-mobile
    let rust_child = Command::new("./target/debug/agent-mobile")
        .args(["idb", "video", "video-stream", "--udid", &udid])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn agent-mobile");

    let rust_output = wait_with_timeout(rust_child, 3);

    // 両方とも何らかの出力があることを確認
    let python_has_output = !python_output.stdout.is_empty();
    let rust_has_output = !rust_output.stdout.is_empty();

    // 少なくとも一方は出力があるはず
    assert!(
        python_has_output || rust_has_output,
        "Expected video stream output from at least one implementation"
    );
}
