# agent-mobile テスト戦略ガイド

agent-mobile CLI開発における3層テスト（ユニットテスト、統合テスト、実機確認）の詳細ガイド。

## 目次

- [テスト戦略概要](#テスト戦略概要)
- [レイヤー1: ユニットテスト](#レイヤー1-ユニットテスト)
- [レイヤー2: 統合テスト](#レイヤー2-統合テスト)
- [レイヤー3: 実機動作確認（必須）](#レイヤー3-実機動作確認必須)
- [テストヘルパー関数](#テストヘルパー関数)
- [TDDサイクル](#tddサイクル)
- [トラブルシューティング](#トラブルシューティング)

## 前提条件: 環境セットアップ

テストを実行する前に、開発環境が正しくセットアップされている必要があります。

### 初回セットアップ

**iOS:**
```bash
./.claude/skills/agent-mobile-dev/scripts/setup-ios.sh
```

**実行内容:**
1. Xcode Command Line Tools確認
2. idb_companionインストール確認
3. シミュレータ起動（未起動の場合）
4. agent-mobile接続確認

**Android:**
```bash
./.claude/skills/agent-mobile-dev/scripts/setup-android.sh
```

**実行内容:**
1. Android SDK (adb, emulator)確認
2. adb server起動
3. エミュレータ起動（未起動の場合）
4. agent-mobile接続確認

### セットアップが必要なタイミング

- **初回セットアップ**: agent-mobile開発を初めて行う場合
- **システムアップデート後**: Xcode、Android Studioなどを更新した場合
- **デバイス検出エラー**: テスト実行時にデバイスが検出されない場合

**詳細は `environment-setup.md` を参照してください。**

---

## テスト戦略概要

### 3層テストピラミッド

```text
              ┌────────────────┐
              │  実機動作確認  │  ← 必須! 視覚的検証
              │   (Manual)     │
              └────────────────┘
                      ↑
             ┌─────────────────┐
             │   統合テスト     │  ← 実デバイス/シミュレータ
             │  (Integration)   │
             └─────────────────┘
                      ↑
          ┌──────────────────────┐
          │   ユニットテスト      │  ← 単体ロジック、モック不要
          │      (Unit)           │
          └──────────────────────┘
```

### 各層の目的

| レイヤー | 目的 | 実行タイミング | 必須度 |
|---------|------|---------------|--------|
| **ユニットテスト** | ロジック検証 | 実装中、コミット前 | 推奨 |
| **統合テスト** | CLI動作検証 | コミット前 | 推奨 |
| **実機確認** | 視覚的動作確認 | コミット前 | **必須** |

**重要**: 実機確認は**必須ステップ**です。ビルドが通っても、実際の動作を確認するまでコミットしないでください。

## レイヤー1: ユニットテスト

### 概要

- **場所**: `src/` 内の `#[cfg(test)] mod tests`
- **目的**: 単体ロジック検証（引数パース、計算、変換など）
- **依存**: 外部デバイス不要
- **実行**: `cargo test --verbose --bins`

### 基本構造

```rust
// src/core/my_feature.rs

pub fn parse_coordinates(input: &str) -> Result<(f64, f64), String> {
    let parts: Vec<&str> = input.split(',').collect();
    if parts.len() != 2 {
        return Err("Invalid format. Use: x,y".into());
    }

    let x: f64 = parts[0].parse().map_err(|_| "Invalid x coordinate")?;
    let y: f64 = parts[1].parse().map_err(|_| "Invalid y coordinate")?;

    Ok((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_coordinates_valid() {
        let result = parse_coordinates("100,200");
        assert_eq!(result, Ok((100.0, 200.0)));
    }

    #[test]
    fn test_parse_coordinates_invalid_format() {
        let result = parse_coordinates("100");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid format. Use: x,y");
    }

    #[test]
    fn test_parse_coordinates_non_numeric() {
        let result = parse_coordinates("abc,def");
        assert!(result.is_err());
    }
}
```

### テスト対象

- **引数パース**: 文字列 → 構造体変換
- **座標計算**: 相対座標、中心計算など
- **バリデーション**: 範囲チェック、形式チェック
- **変換ロジック**: データ変換、フォーマット変換
- **エラーメッセージ**: 期待通りのエラーが返されるか

### ベストプラクティス

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // テスト名は明確に
    #[test]
    fn test_parse_target_element_ref() {
        let target = Target::parse("@e1");
        assert_eq!(target, Target::Ref(1));
    }

    #[test]
    fn test_parse_target_text() {
        let target = Target::parse("\"Login\"");
        assert_eq!(target, Target::Text("Login".to_string()));
    }

    #[test]
    fn test_parse_target_coords() {
        let target = Target::parse("100,200");
        assert_eq!(target, Target::Coords(100.0, 200.0));
    }

    // エラーケースも必ずテスト
    #[test]
    fn test_parse_target_invalid_ref() {
        let target = Target::parse("@e");
        assert_eq!(target, Target::Invalid);
    }
}
```

### 実行コマンド

```bash
# 全ユニットテスト実行
cargo test --verbose --bins

# 特定ファイルのみ
cargo test --verbose --bin agent-mobile my_feature

# 特定テストのみ
cargo test --verbose --bin agent-mobile test_parse_coordinates_valid
```

## レイヤー2: 統合テスト

### 概要

- **場所**: `tests/cli/<name>_integration.rs`
- **目的**: CLIコマンド全体の動作検証
- **依存**: 実デバイス/シミュレータ、idb_companion
- **実行**: `cargo test --test cli -- --test-threads=1`

### 基本構造

```rust
// tests/cli/my_feature_integration.rs

use std::process::Output;

#[path = "common/mod.rs"]
mod common;

/// Run my_feature command
fn run_command(args: &[&str]) -> Output {
    common::run_cli_command("my-feature", args)
}

/// Run my_feature command with UDID
fn run_command_with_udid(args: &[&str], udid: &str) -> Output {
    common::run_cli_command_with_udid("my-feature", args, udid)
}

/// Test success case
#[test]
fn test_my_feature_success() {
    // 1. 利用可能なデバイス取得
    let udid = common::get_available_udid();

    // 2. idb_companion起動確認
    common::ensure_companion_running(&udid);

    // 3. コマンド実行
    let output = run_command_with_udid(&[], &udid);

    // 4. 成功確認
    common::assert_success(&output, "my_feature command");

    // 5. 出力検証
    let stdout = common::get_stdout(&output);
    assert!(stdout.contains("expected text"));
}

/// Test with invalid device
#[test]
fn test_my_feature_invalid_device() {
    let output = run_command_with_udid(&[], "invalid-udid-12345");

    // 失敗を期待
    common::assert_failure(&output, "Invalid UDID");

    // エラーメッセージ検証
    let stderr = common::get_stderr(&output);
    assert!(
        stderr.contains("device") || stderr.contains("UDID"),
        "Error message should mention device/UDID: {}",
        stderr
    );
}

/// Test with custom arguments
#[test]
fn test_my_feature_with_args() {
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let output = run_command_with_udid(&["--arg", "value"], &udid);

    common::assert_success(&output, "my_feature with custom arg");
}
```

### テストパターン

#### 正常系テスト

```rust
#[test]
fn test_tap_at_coords() {
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    // 座標指定でタップ
    let output = run_command_with_udid(&["100,200"], &udid);
    common::assert_success(&output, "tap at coordinates");
}
```

#### 異常系テスト

```rust
#[test]
fn test_tap_invalid_coords() {
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    // 不正な座標
    let output = run_command_with_udid(&["invalid"], &udid);
    common::assert_failure(&output, "Invalid coordinates");
}
```

#### JSON出力テスト

```rust
#[test]
fn test_find_json_output() {
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let output = run_command_with_udid(&["Login", "--format", "json"], &udid);
    common::assert_success(&output, "find with JSON output");

    // JSON検証
    let json = common::assert_valid_json(&output);
    assert_eq!(json["status"], "success");
    assert!(json["elements"].is_array());
}
```

#### 複数引数テスト

```rust
#[test]
fn test_swipe_with_duration() {
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let output = run_command_with_udid(
        &["up", "--duration", "0.5"],
        &udid
    );
    common::assert_success(&output, "swipe with duration");
}
```

### 実行コマンド

```bash
# 前提: シミュレータ起動 + idb_companion起動

# 全統合テスト実行
cargo test --test cli -- --test-threads=1

# 特定機能のみ
cargo test --test cli my_feature -- --test-threads=1

# 特定テストのみ
cargo test --test cli test_my_feature_success -- --test-threads=1
```

### トラブルシューティング

**テストが失敗する場合**:

1. シミュレータが起動しているか確認
   ```bash
   xcrun simctl list devices | grep Booted
   ```

2. idb_companionが起動しているか確認
   ```bash
   cat /tmp/idb/state
   ```

3. バイナリがビルドされているか確認
   ```bash
   cargo build
   ls -la target/debug/agent-mobile
   ```

## レイヤー3: 実機動作確認（必須）

### 概要

**⚠️ 最重要**: ビルド成功≠正しい動作

- **場所**: 実デバイス/シミュレータ
- **目的**: 視覚的な動作確認、UI検証
- **実行**: `/mobile-e2e ios` または `/mobile-e2e android`
- **必須度**: **100%必須** - 確認なしでのコミットは禁止

### iOS確認手順

#### 1. テスト環境起動

```bash
/mobile-e2e ios
```

**実行内容**:
- 適切なシミュレータを起動（未起動の場合）
- テストアプリをインストール（未インストールの場合）
- idb_companionを起動（未起動の場合）
- 環境情報を表示（UDID、シミュレータ名など）

#### 2. コマンド実行（正常系）

```bash
# 例: tapコマンドの確認
agent-mobile tap 100,200 --udid <udid>

# 例: swipeコマンドの確認
agent-mobile swipe up --udid <udid>

# 例: findコマンドの確認
agent-mobile find "Login" --udid <udid>
```

#### 3. シミュレータで視覚確認

- **UI操作が期待通りか**: タップ、スワイプが正しい位置で実行されるか
- **画面遷移が正しいか**: 期待する画面に遷移するか
- **アニメーションが適切か**: 動きが自然か
- **エラー表示が適切か**: エラー時に適切なメッセージが表示されるか

#### 4. 証跡保存

```bash
# スクリーンショットで証跡保存
agent-mobile screenshot /tmp/my_feature_evidence.png --udid <udid>

# 画像確認
open /tmp/my_feature_evidence.png
```

#### 5. 異常系確認

```bash
# 不正なUDID
agent-mobile my-feature --udid invalid-udid
# → エラーメッセージが明確で実行可能か確認

# 不正な引数
agent-mobile my-feature invalid-arg --udid <udid>
# → エラーメッセージが明確か確認

# 不正な座標
agent-mobile tap 99999,99999 --udid <udid>
# → 適切にエラーハンドリングされるか確認
```

### Android確認手順

#### 1. テスト環境起動

```bash
/mobile-e2e android
```

**実行内容**:
- エミュレータを起動（未起動の場合）
- テストアプリをインストール（未インストールの場合）
- 環境情報を表示（シリアル番号など）

#### 2. コマンド実行

```bash
# 例: tapコマンドの確認
agent-mobile tap 100,200 --platform android

# 例: swipeコマンドの確認
agent-mobile swipe up --platform android
```

#### 3. エミュレータで視覚確認

iOS同様、視覚的な動作確認を実施。

#### 4. 証跡保存

```bash
agent-mobile screenshot /tmp/my_feature_android.png --platform android
open /tmp/my_feature_android.png
```

### 確認チェックリスト

実機確認時に必ず確認する項目:

#### 正常系

- [ ] コマンドがエラーなく実行される
- [ ] UI操作が期待通りの位置/動作で実行される
- [ ] 画面遷移が正しい
- [ ] アニメーションが自然
- [ ] 出力メッセージが適切（verbose、わかりやすい）
- [ ] JSON出力が正しい構造（--format json の場合）

#### 異常系

- [ ] 不正なUDIDでエラーメッセージが明確
- [ ] 不正な引数でエラーメッセージが明確
- [ ] エラーメッセージに次のアクションが含まれる
- [ ] エラー時にクラッシュしない
- [ ] エラー時のJSON出力が構造化されている

#### 追加確認（該当する場合）

- [ ] Python idbとの動作差異なし
- [ ] 複数デバイスで動作確認
- [ ] 長時間実行時のメモリリークなし
- [ ] ストリーミング時のCtrl+C処理が正常

### 実機確認の重要性

**なぜ必須か**:

1. **視覚的な検証**: UI操作は視覚的に確認しないと正しさが判断できない
2. **実環境での動作**: シミュレータと実機で挙動が異なる場合がある
3. **エラーメッセージ**: 実際のユーザーが見るエラーメッセージを確認
4. **パフォーマンス**: 実行速度、レスポンスが適切か
5. **統合動作**: 他の機能との統合動作が正しいか

**実機確認なしでのコミットは禁止**: ビルドが通っても、実際の動作確認までコミットしないでください。

## テストヘルパー関数

### tests/cli/common/mod.rs

CLI統合テスト用のヘルパー関数。

#### デバイス管理

```rust
/// 利用可能なUDIDを取得
pub fn get_available_udid() -> String

/// idb_companionが起動していることを確認
pub fn ensure_companion_running(udid: &str)

/// テストバンドルIDを取得
pub fn get_test_bundle_id() -> String
```

**使用例**:
```rust
let udid = common::get_available_udid();
common::ensure_companion_running(&udid);
```

#### コマンド実行

```rust
/// agent-mobile CLIコマンドを実行
pub fn run_cli_command(feature: &str, args: &[&str]) -> Output

/// agent-mobile CLIコマンドを実行（UDID付き）
pub fn run_cli_command_with_udid(feature: &str, args: &[&str], udid: &str) -> Output
```

**使用例**:
```rust
// agent-mobile tap 100,200 --udid <udid>
let output = common::run_cli_command_with_udid("tap", &["100,200"], &udid);
```

#### アサーション

```rust
/// コマンド成功を確認
pub fn assert_success(output: &Output, context: &str)

/// コマンド失敗を確認
pub fn assert_failure(output: &Output, context: &str)

/// 有効なJSONであることを確認
pub fn assert_valid_json(output: &Output) -> serde_json::Value

/// 標準出力に特定のテキストが含まれることを確認
pub fn assert_stdout_contains(output: &Output, expected: &str)

/// 標準エラーに特定のテキストが含まれることを確認
pub fn assert_stderr_contains(output: &Output, expected: &str)
```

**使用例**:
```rust
let output = run_command_with_udid(&[], &udid);
common::assert_success(&output, "tap command");
common::assert_stdout_contains(&output, "Tapped at");
```

#### 出力取得

```rust
/// 標準出力を文字列として取得
pub fn get_stdout(output: &Output) -> String

/// 標準エラーを文字列として取得
pub fn get_stderr(output: &Output) -> String
```

**使用例**:
```rust
let stdout = common::get_stdout(&output);
assert!(stdout.contains("expected text"));
```

### tests/idb/common/mod.rs

idb gRPC統合テスト用のヘルパー関数（CLIテストでも使用可能）。

#### クライアント管理

```rust
/// IdbClient取得
pub async fn get_client() -> Result<IdbClient>

/// 利用可能なUDID取得
pub fn get_available_udid() -> String

/// idb_companion起動確認
pub fn ensure_companion_running(udid: &str)
```

#### テストアプリ操作

```rust
/// テストアプリ起動
pub async fn launch_test_app(client: &IdbClient) -> Result<()>

/// テストアプリ終了
pub async fn terminate_test_app(client: &IdbClient) -> Result<()>
```

#### メディア操作

```rust
/// スクリーンショット取得
pub async fn take_screenshot(client: &IdbClient) -> Result<Vec<u8>>

/// Framebufferが準備完了していることを確認
pub async fn ensure_framebuffer_ready(client: &IdbClient) -> Result<()>
```

#### ファイル操作

```rust
/// ファイルをプッシュ
pub async fn push_file(client: &IdbClient, src: &str, dst: &str) -> Result<()>

/// ファイルをプル
pub async fn pull_file(client: &IdbClient, src: &str) -> Result<Vec<u8>>
```

## TDDサイクル

### テスト駆動開発の推奨フロー

```text
1. テスト作成 → 2. 実装 → 3. リファクタ → 4. 実機確認
     ↑                                           │
     └───────────────────────────────────────────┘
```

#### フェーズ1: テスト作成（Red）

```bash
# 1. 統合テストを先に書く
# tests/cli/vibrate_integration.rs

#[test]
fn test_vibrate_success() {
    let udid = common::get_available_udid();
    common::ensure_companion_running(&udid);

    let output = run_command_with_udid(&[], &udid);
    common::assert_success(&output, "vibrate command");
}

# 2. 実行 → 失敗（コマンドが未実装）
cargo test --test cli vibrate
# → FAILED
```

#### フェーズ2: 実装（Green）

```bash
# 3. 最小限の実装
./scripts/new-command.sh vibrate

# 4. vibrate.rsを実装
# src/core/vibrate.rs
# ...

# 5. ビルド
cargo build

# 6. テスト実行 → 成功
cargo test --test cli vibrate -- --test-threads=1
# → PASSED
```

#### フェーズ3: リファクタ（Refactor）

```bash
# 7. コード改善
# - 重複削除
# - 関数分割
# - エラーハンドリング改善

# 8. テスト再実行 → 成功維持
cargo test --test cli vibrate -- --test-threads=1
# → PASSED
```

#### フェーズ4: 実機確認（Verify）

```bash
# 9. 実機で動作確認（必須!）
/mobile-e2e ios

agent-mobile vibrate --udid <udid>
# → シミュレータでバイブレーション動作を視覚確認

# 10. 証跡保存
agent-mobile screenshot /tmp/vibrate_evidence.png --udid <udid>

# 11. 異常系確認
agent-mobile vibrate --udid invalid-udid
# → エラーメッセージが適切か確認
```

### TDDのメリット

- **仕様が明確**: テストが仕様書になる
- **リファクタが安全**: テストが品質を保証
- **バグ早期発見**: 実装中にバグを発見
- **ドキュメント**: テストが使用例になる

## トラブルシューティング

### 統合テストが失敗する

#### 症状: "Failed to run agent-mobile"

**原因**: バイナリがビルドされていない

**解決**:
```bash
cargo build
ls -la target/debug/agent-mobile  # 確認
```

#### 症状: "No available UDID"

**原因**: シミュレータが起動していない

**解決**:
```bash
# シミュレータ一覧確認
xcrun simctl list devices | grep Booted

# シミュレータ起動
xcrun simctl boot <udid>

# または
/mobile-e2e ios
```

#### 症状: "Failed to connect to idb_companion"

**原因**: idb_companionが起動していない

**解決**:
```bash
# companion状態確認
cat /tmp/idb/state

# companion起動
idb_companion --udid <udid> &

# または
/mobile-e2e ios
```

### 実機確認でコマンドが動作しない

#### 症状: タップが意図した位置をタップしない

**確認**:
1. 座標が正しいか確認
2. 画面の向き（portrait/landscape）を確認
3. スクリーンショットで座標を確認

**デバッグ**:
```bash
# アクセシビリティ情報取得
agent-mobile idb accessibility-describe-all --udid <udid>

# 要素の座標を確認
agent-mobile find "Login" --format json --udid <udid>
```

#### 症状: エラーメッセージが表示されない

**確認**:
1. stderrにエラーが出力されているか
2. JSON出力の場合、エラーが構造化されているか

**デバッグ**:
```bash
# 詳細なエラー出力
RUST_BACKTRACE=1 agent-mobile my-command --udid <udid>
```

## まとめ

**開発フロー**:
1. **ユニットテスト**: `cargo test --verbose --bins`
2. **統合テスト**: `cargo test --test cli -- --test-threads=1`
3. **実機確認**: `/mobile-e2e ios` で視覚的に検証（必須!）
4. **コミット**: 全テストパス + 実機確認完了後

**実機確認は必須**: ビルド成功≠正しい動作

**テストの質**:
- 正常系だけでなく異常系もテスト
- エラーメッセージの明確性を確認
- 視覚的な動作を実機で確認

**関連ファイル**:
- `tests/cli/common/mod.rs`: CLIテストヘルパー
- `tests/idb/common/mod.rs`: idbテストヘルパー
- `assets/checklist.md`: 実装チェックリスト
