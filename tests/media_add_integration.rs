mod common;

use std::process::Command;

#[test]
#[ignore]
fn test_media_add_basic() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    // Create a small test image (1x1 PNG)
    let test_image_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44,
        0x41, // IDAT chunk
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD,
        0x8D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND chunk
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let temp_file = std::env::temp_dir().join("test_media_image.png");
    std::fs::write(&temp_file, &test_image_data).expect("Failed to create test image");

    // Run Python idb media add-media
    let python_output = Command::new("idb")
        .args([
            "media",
            "add-media",
            temp_file.to_str().unwrap(),
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run Python idb");

    // Run agent-mobile media add-media
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "media",
            "add-media",
            temp_file.to_str().unwrap(),
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // Compare exit codes
    assert_eq!(
        python_output.status.code(),
        rust_output.status.code(),
        "Exit codes differ:\n  Python: {:?}\n  Rust: {:?}",
        python_output.status.code(),
        rust_output.status.code()
    );

    // Both should succeed
    assert!(
        python_output.status.success(),
        "Python idb failed: {}",
        String::from_utf8_lossy(&python_output.stderr)
    );
    assert!(
        rust_output.status.success(),
        "agent-mobile failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );

    // Cleanup
    let _ = std::fs::remove_file(temp_file);
}

#[test]
#[ignore]
fn test_media_add_multiple_files() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    // Create multiple test images
    let test_image_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB4, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let temp_file1 = std::env::temp_dir().join("test_media_image1.png");
    let temp_file2 = std::env::temp_dir().join("test_media_image2.png");

    std::fs::write(&temp_file1, &test_image_data).expect("Failed to create test image 1");
    std::fs::write(&temp_file2, &test_image_data).expect("Failed to create test image 2");

    // Run agent-mobile media add-media with multiple files
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "media",
            "add-media",
            temp_file1.to_str().unwrap(),
            temp_file2.to_str().unwrap(),
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // Should succeed
    assert!(
        rust_output.status.success(),
        "agent-mobile failed: {}",
        String::from_utf8_lossy(&rust_output.stderr)
    );

    // Cleanup
    let _ = std::fs::remove_file(temp_file1);
    let _ = std::fs::remove_file(temp_file2);
}

#[test]
#[ignore]
fn test_media_add_nonexistent_file() {
    common::build_agent_mobile();
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let nonexistent_file = "/tmp/nonexistent_media_file_12345.png";

    // Run Python idb media add-media
    let python_output = Command::new("idb")
        .args(["media", "add-media", nonexistent_file, "--udid", &udid])
        .output()
        .expect("Failed to run Python idb");

    // Run agent-mobile media add-media
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args([
            "idb",
            "media",
            "add-media",
            nonexistent_file,
            "--udid",
            &udid,
        ])
        .output()
        .expect("Failed to run agent-mobile");

    // Both should fail
    assert!(!python_output.status.success());
    assert!(!rust_output.status.success());
}
