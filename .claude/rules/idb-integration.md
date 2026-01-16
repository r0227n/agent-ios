---
paths:
  - "tests/**.rs"
---

# Integration Test Rules

## Python idb 必須

- **Python idb を必須とする**: テスト実行時に `idb` コマンドが利用可能であることを前提とする
- `is_python_idb_available()` のようなスキップロジックは使用しない
- Python idb がない環境ではテストが失敗するのが正しい動作

## エラーハンドリング

- **コマンド実行失敗時は異常終了**: `.ok()?` でスキップせず、`.expect()` でパニックする
- **ステータス確認**: `output.status.success()` を `assert!` でチェックする

```rust
// Good
let output = Command::new("idb")
    .args(["list-targets", "--json"])
    .output()
    .expect("Failed to execute Python idb - ensure idb is installed");

assert!(
    output.status.success(),
    "Python idb failed: {}",
    String::from_utf8_lossy(&output.stderr)
);

// Bad - 静かにスキップしてしまう
let output = Command::new("idb").output().ok()?;
if !output.status.success() {
    return None;
}
```

## get_available_udid() パターン

- 戻り値は `String`（`Option<String>` ではない）
- Python idb の `list-targets --json` を使用して UDID を取得
- Booted 状態のシミュレータがない場合は `panic!`

```rust
fn get_available_udid() -> String {
    let output = Command::new("idb")
        .args(["list-targets", "--json"])
        .output()
        .expect("Failed to execute Python idb");

    assert!(output.status.success(), "idb list-targets failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if json.get("state").and_then(|v| v.as_str()) == Some("Booted") {
                return json.get("udid").unwrap().as_str().unwrap().to_string();
            }
        }
    }
    panic!("No booted simulator available");
}
```

## テスト関数での使用

- `match get_available_udid()` のスキップパターンは使用しない
- 直接 `let udid = get_available_udid();` で取得

```rust
// Good
#[test]
fn test_example() {
    let udid = get_available_udid();
    // ...
}

// Bad - スキップロジック
#[test]
fn test_example() {
    let udid = match get_available_udid() {
        Some(u) => u,
        None => {
            eprintln!("Skipping test");
            return;
        }
    };
}
```