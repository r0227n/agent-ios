---
name: development-guide
description: Guide for developing CLI commands in agent-mobile (Rust mobile E2E testing tool). Use when adding new commands, implementing features, or refactoring agent-mobile code. Enforces 4-layer architecture (CLI/Gateway/Platform/Core), platform decision-making (iOS: XCUITest Runner HTTP vs xcrun simctl, Android: ADB native protocol), and mandatory real-device testing workflow (Phase 0 environment setup → design → implement → test → device verification → commit).
version: 3.0.0
argument-hint: "[setup-ios|setup-android|<other-args>]"
---

# agent-mobile CLI開発スキル

agent-mobile CLI（Rust製モバイルE2Eテストツール）の新機能開発、既存機能リファクタリングのための包括的ガイド。4層アーキテクチャの理解、プラットフォーム実装判断、必須開発フローを体系化しています。

## 概要

### agent-mobileとは

**agent-mobile**は、iOS/Android両対応のRust製モバイル自動化CLIツールです。AI開発者が効率的に使えるように設計されています。

**主要特徴:**
- **4層アーキテクチャ**: CLI → Gateway → Platform → Core
- **iOS対応**: XCUITest Runner (HTTP, localhost:8200) + xcrun simctl ハイブリッド
- **Android対応**: ADB native protocol (TCP :5037) via adb_client crate
- **AI最適化**: 簡潔なコマンド、JSON出力、エラーメッセージ明確化

**技術スタック:**
```
Rust 2021 | tokio 1.49 | reqwest (HTTP) | clap 4.5 | adb_client 2.0
```

### 現在のCLIコマンド一覧

```text
Core Commands (AI Agent 向け):
  tap          - Tap an element by ref, text, coordinates, or key
  long-press   - Long press with configurable duration
  check        - Idempotent checkbox check
  uncheck      - Idempotent checkbox uncheck
  select       - Select from picker/spinner
  fill         - Clear + type text into field
  type         - Append text to focused field
  swipe        - Swipe gesture (up/down/left/right)
  scroll       - Scroll within element or screen
  get          - Get element property
  is           - Check element state (exit code)
  wait         - Wait for element appearance
  find         - Semantic locator search with actions
  screenshot   - Take screenshot
  snapshot     - Capture UI with element references (@e1, @e2...)
  record       - Record screen to MP4
  console      - Stream device logs

Management Commands:
  app          - App management (launch, terminate, install, list)
  device       - Device management (list, boot, shutdown)
  session      - Session management (list, show, create, destroy)
  doctor       - Check external dependency availability

Global Options:
  --session <name>  - Use named session (also AGENT_MOBILE_SESSION env)
```

### 4層アーキテクチャ

```text
┌─────────────────────────────────────────────┐
│  CLI Layer (src/)                           │
│  - コマンドパース (clap derive)               │
│  - 引数定義 (src/command.rs)                 │
│  - コア操作 (src/core/*.rs)                  │
│  - スナップショット (src/snapshot/)           │
│  - セッション管理 (src/session/)              │
│  - 管理操作 (src/app.rs, device.rs, etc.)    │
│  - ヘルパー (src/helpers/)                   │
└───────────────────┬─────────────────────────┘
                    │
                    ↓
┌─────────────────────────────────────────────┐
│  Gateway Layer (crates/gateway/)            │
│  - DeviceResolver (プラットフォーム検出)      │
│  - AndroidDevice (Android統一API)           │
│  - stream_console_logs (共通ログ)           │
└──────┬──────────────────────┬───────────────┘
       │                      │
       ↓ (iOS)                ↓ (Android)
┌────────────────┐      ┌──────────────────┐
│ iOS Platform   │      │ Android Platform │
│ - XCUITest     │      │ - ADB native     │
│   (HTTP)       │      │   (TCP :5037)    │
│ - simctl       │      │ - uiautomator    │
│ - coresim(FFI) │      │ - snapshot       │
│ - snapshot     │      └──────────────────┘
└────────────────┘
┌─────────────────────────────────────────────┐
│  Core Layer (crates/core/)                  │
│  - Platform enum                            │
│  - OutputWriter (stdout/file/tee)           │
│  - Error types, Traits                      │
│  - Snapshot types (Frame, RawElement)        │
│  - Types (TargetType, Address, ScrollDir)    │
└─────────────────────────────────────────────┘
```

**各層の責務:**
- **CLI層**: コマンドパース、引数検証、プラットフォーム分岐
- **Gateway層**: プラットフォーム検出（DeviceResolver）、Android統一API
- **Platform層**: iOS (XCUITest Runner HTTP / simctl)、Android (ADB native protocol) 実装
- **Core層**: 共通型（Platform enum）、OutputWriter、エラー型、トレイト

## Quick Start: 新機能追加の判断フロー

新しいCLIコマンドを追加する際の判断フローチャート:

```text
1. 機能の分類
   ├─ コア操作（tap, swipe, find等）？
   │  └─ src/core/<name>.rs に実装
   └─ 管理操作（app, device, session等）？
      └─ src/<name>.rs に実装

2. iOS実装判断
   ├─ UI操作/要素操作/スクリーンショット？
   │  └─ XCUITest Runner (HTTP) で実装
   │     └─ with_xcuitest() パターン使用
   ├─ デバイスライフサイクル（boot/shutdown/create/delete）？
   │  └─ xcrun simctl で実装
   │     └─ simctl::management モジュール拡張
   └─ 両方で可能？
      └─ ハイブリッド (XCUITest Runner優先、fallback)

3. Android実装判断
   ├─ ADB native protocol で実装可能？
   │  └─ crates/platform-android/src/adb/ に実装
   └─ 複雑な操作？
      └─ Gateway層の AndroidDevice 拡張

4. テスト戦略
   ├─ ユニットテスト: src/ 内の #[cfg(test)] mod tests
   ├─ 統合テスト: tests/cli/<name>_integration.rs
   └─ 実機確認: /mobile-e2e ios or /mobile-e2e android (必須!)
```

### 判断支援ツール

**実装方法判断**（iOS実装判断用）:
```bash
./scripts/platform-check.sh <feature-name>
# → XCUITest Runner or simctl の推奨実装を提示
```

**コマンドテンプレート生成**:
```bash
./scripts/new-command.sh <command-name>
# → src/core/<name>.rs と tests/cli/<name>_integration.rs を自動生成
```

## Phase 0: 環境セットアップ（初回・プラットフォーム切替時）

機能開発を開始する前に、開発環境が正しくセットアップされていることを確認してください。

### 環境診断（推奨）

```bash
# doctor コマンドで環境を一括チェック
cargo run -- doctor

# JSON出力で確認
cargo run -- doctor --format json
```

**チェック項目:**
- iOS: Xcode, simctl, CoreSimulator, XCUITest Runner
- Android: ADB server, Android SDK, Android Emulator

### iOS環境セットアップ

```bash
./scripts/setup-ios.sh
```

**実行内容:**
1. Xcode & xcrun simctl確認
2. XCUITest Runnerインストール確認
3. シミュレータ起動（未起動の場合）
4. agent-mobile接続確認

### Android環境セットアップ

```bash
./scripts/setup-android.sh
```

**実行内容:**
1. Android SDK (adb, emulator)確認
2. adb server起動
3. エミュレータ起動（未起動の場合）
4. agent-mobile接続確認

### セットアップが必要なタイミング

- **初回セットアップ**: agent-mobile開発を初めて行う場合
- **システムアップデート後**: Xcode、Android Studioなどを更新した場合
- **プラットフォーム切り替え**: iOS ↔ Android開発を切り替える場合
- **デバイス検出エラー**: `agent-mobile device list` でデバイスが検出されない場合

**詳細ガイド**: `references/environment-setup.md` 参照

## 開発ワークフロー（必須6ステップ）

**Phase 0 → Design → Implement → Unit Test → Integration Test → Real Device Verification → Commit**

**重要**: 実機確認はビルド成功後の**必須ステップ**です。ビルドが通っても実際の動作を確認するまでコミットしないでください。

### ステップ1: 設計

**プラットフォーム実装判断:**
1. iOS: XCUITest Runner or simctl の判断
   ```bash
   ./scripts/platform-check.sh <feature>
   # → XCUITest Runner (HTTP) or xcrun simctl の推奨を提示
   ```
2. Android: ADB native protocolで実現可能か確認

**引数設計:**
```rust
#[derive(Args, Debug)]
pub struct MyCommandArgs {
    /// Target: @eN ref, "text", or coordinates
    pub target: String,

    #[command(flatten)]
    pub device: DeviceArgs,  // --udid のみ (platformは自動検出)
}
```

### ステップ2: 実装

**コマンドテンプレート使用:**
```bash
./scripts/new-command.sh vibrate
# 生成:
# ✓ src/core/vibrate.rs
# ✓ tests/cli/vibrate_integration.rs
# ⚠ src/command.rs に手動追加が必要
```

**手動実装の場合:**
```rust
// src/core/my_feature.rs
use clap::Args;

use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

use crate::helpers::client::{with_xcuitest, CommandResult};
use crate::helpers::common_args::DeviceArgs;

#[derive(Args, Debug)]
pub struct MyFeatureArgs {
    #[command(flatten)]
    pub device: DeviceArgs,
}

pub async fn run(args: MyFeatureArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    match platform {
        Platform::Ios => run_ios().await,
        Platform::Android => run_android().await,
    }
}

async fn run_ios() -> CommandResult {
    with_xcuitest(|client| async move {
        // XCUITest Runner (HTTP)実装
        Ok(())
    }).await
}

async fn run_android() -> CommandResult {
    // ADB native protocol実装
    Ok(())
}
```

**src/command.rsに追加:**
```rust
// Commands enum にバリアント追加
#[derive(Subcommand)]
pub enum Commands {
    // ...
    MyFeature(crate::core::my_feature::MyFeatureArgs),
}
```

**src/main.rs の match 分岐に追加:**
```rust
Commands::MyFeature(args) => crate::core::my_feature::run(args).await,
```

### ステップ3: ユニットテスト

```bash
# ビルド検証
cargo build --verbose

# ユニットテスト実行
cargo test --verbose --bins
```

**ユニットテストの書き方:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target() {
        let target = Target::parse("@e1");
        assert!(matches!(target, Target::Ref(1)));
    }
}
```

### ステップ4: 統合テスト

```bash
# 統合テスト実行（シミュレータ/エミュレータ必要）
cargo test --test cli -- --test-threads=1
```

**統合テストの書き方:**
```rust
// tests/cli/my_feature_integration.rs
use crate::common::{
    assert_success, ensure_device_ready, get_available_udid,
    run_cli_command_with_udid,
};

#[test]
fn test_my_feature_success() {
    let udid = get_available_udid();
    ensure_device_ready(&udid);

    let output = run_cli_command_with_udid("my-feature", &[], &udid);
    assert_success(&output, "my-feature command");
}

#[test]
fn test_my_feature_no_device() {
    let output = run_cli_command_with_udid("my-feature", &[], "invalid-udid");
    // 失敗を期待
}
```

### ステップ5: 実機動作確認（必須!）

**iOS確認手順:**
```bash
# 1. テスト環境起動
/mobile-e2e ios

# 2. コマンド実行
agent-mobile my-feature --udid <udid>

# 3. シミュレータで結果を視覚確認

# 4. 証跡保存
agent-mobile screenshot /tmp/evidence.png

# 5. 異常系も確認
agent-mobile my-feature --udid invalid-udid
# → エラーメッセージが適切か確認
```

**Android確認手順:**
```bash
# 1. テスト環境起動
/mobile-e2e android

# 2. コマンド実行
agent-mobile my-feature --udid <udid>

# 3. エミュレータで結果を視覚確認

# 4. 証跡保存
agent-mobile screenshot /tmp/evidence.png
```

**チェックリスト:**
- [ ] コマンドが期待通り動作（正常系）
- [ ] エラーメッセージが適切（異常系）
- [ ] UI操作結果が視覚的に確認可能
- [ ] スクリーンショットで証跡保存

**実機確認なしでのコミットは禁止**: ビルドが通っても、実際の動作確認までコミットしないでください。

### ステップ6: コミット

```bash
git add .
git commit -m "feat: add my-feature command"
```

## 頻出パターン (Quick Reference)

### with_xcuitest() パターン

最も頻繁に使用するパターン。XCUITest Runnerへの接続を抽象化します。
Runner の自動起動・ヘルスチェックを内部で行います。

**基本形:**
```rust
pub async fn run() -> CommandResult {
    with_xcuitest(|client| async move {
        // XCUITest Runner (HTTP)操作
        client.accessibility_info(None, true).await?;
        Ok(())
    }).await
}
```

**重要**: `with_xcuitest()` はUDIDパラメータを取りません。デバイス選択はCLI層で行い、Platform分岐後に呼び出します。

### プラットフォーム検出パターン

```rust
// UDIDが指定されている場合 → UDIDからプラットフォームを判定
// UDIDが未指定の場合 → 起動中のデバイスから自動検出
let platform = match args.device.udid.as_deref() {
    Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
    None => DeviceResolver::detect_platform().await?,
};
```

**DeviceResolver::detect_platform()** の検出順序:
1. iOS: `simctl::list_simulators()` で Booted シミュレータ確認
2. Android: ADB native protocol でデバイス確認
3. どちらもなければエラー

### DeviceArgs flatten パターン

デバイス指定引数（--udid）を標準化します。プラットフォームはUDIDから自動検出されます。

```rust
#[derive(Args, Debug)]
pub struct MyCommandArgs {
    /// コマンド固有の引数
    pub target: String,

    #[command(flatten)]
    pub device: DeviceArgs,  // --udid を自動追加
}
```

**バリエーション:**
- `DeviceArgs`: `--udid` のみ
- `FormatArgs`: `--format` のみ
- `FormatOutputArgs`: `--format` + `--output`
- `DeviceFormatArgs`: `--udid` + `--format`

### エラーハンドリング

**CommandResult型:**
```rust
pub type CommandResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
```

**使用例:**
```rust
pub async fn run(args: MyArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    if platform == Platform::Ios {
        with_xcuitest(|client| async move {
            client.focus().await?;
            Ok(())
        }).await
    } else {
        Err("Android not supported".into())
    }
}
```

### JSON出力対応

```rust
#[derive(Args, Debug)]
pub struct MyCommandArgs {
    #[command(flatten)]
    pub device: DeviceFormatArgs,  // --udid, --format
}

pub async fn run(args: MyCommandArgs) -> CommandResult {
    let result = perform_operation().await?;

    match args.device.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "status": "success",
                "data": result
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Text => {
            println!("Result: {}", result);
        }
    }
    Ok(())
}
```

## プラットフォーム判断（iOS）

### 判断フローチャート

```text
機能のカテゴリは？
├─ UI操作（tap/swipe/text/find等）
│  └─ XCUITest Runner (HTTP) 実装
│     └─ with_xcuitest() 使用
├─ 要素情報取得（accessibility/screenshot等）
│  └─ XCUITest Runner (HTTP) 実装
│     └─ with_xcuitest() 使用
├─ アプリ操作（launch/terminate/install等）
│  └─ XCUITest Runner (HTTP) 実装
│     └─ with_xcuitest() 使用
├─ クリップボード操作
│  └─ XCUITest Runner (HTTP) 実装
│     └─ UIPasteboard.general 経由
└─ デバイスライフサイクル（boot/shutdown/create/delete）
   └─ xcrun simctl 実装
      └─ simctl::management モジュール
```

### 機能別推奨実装

| 機能カテゴリ | 推奨実装 | 理由 |
|------------|---------|------|
| HID入力 (tap/swipe/text) | **XCUITest Runner (HTTP)** | 精密な制御 |
| アクセシビリティ | **XCUITest Runner (HTTP)** | simctlでは不可 |
| スクリーンショット | **XCUITest Runner (HTTP)** | 高品質キャプチャ |
| アプリ操作 (launch/terminate) | **XCUITest Runner (HTTP)** | 統一インターフェース |
| クリップボード | **XCUITest Runner (HTTP)** | UIPasteboard.general 経由 |
| シミュレータ起動/停止 | **xcrun simctl** | デバイスライフサイクル管理 |
| シミュレータ作成/削除 | **xcrun simctl** | デバイスライフサイクル管理 |
| デバイス一覧 | **xcrun simctl** | list_simulators() (キャッシュ付き, TTL 5秒) |

### ハイブリッド実装例

XCUITest Runnerを試して、失敗時にsimctlにフォールバック:

```rust
pub async fn grant_permission(bundle: &str, perm: Permission) -> Result<()> {
    // 1. まず XCUITest Runner (HTTP) を試す
    match xcuitest_client.approve(bundle, perm).await {
        Ok(_) => Ok(()),
        Err(e) if e.is_schema_error() => {
            // 2. 失敗時は simctl にフォールバック
            simctl::privacy_grant(udid, perm, bundle)
        }
        Err(e) => Err(e),
    }
}
```

## プラットフォーム判断（Android）

### ADB native protocol 構成

```text
Android操作: ADB native protocol (TCP :5037) via adb_client crate
  ├─ AdbConnection: shell_command, pull, push, install, uninstall
  ├─ Screenshot: screencap -p (PNG直接取得)
  ├─ UI階層: uiautomator dump (XML解析)
  ├─ Input: input tap/swipe/text
  ├─ App: am start/force-stop, pm install/uninstall
  └─ AVD一覧: ~/.android/avd/*.ini ファイル読取
残存CLI: emulator -avd (QEMU起動), adb logcat (ストリーミング)
```

### Android実装例

```rust
use agent_mobile_platform_android::adb;

async fn run_android() -> CommandResult {
    // ADB native protocol でスクリーンショット取得
    let png_data = adb::screenshot::capture(serial).await?;

    // shell コマンド実行
    let output = adb::commands::shell_command(serial, "input tap 100 200").await?;

    Ok(())
}
```

## スクリプト & テンプレート

### scripts/new-command.sh

新しいCLIコマンドの骨格を自動生成します。

**使用方法:**
```bash
./scripts/new-command.sh vibrate
```

**生成内容:**
- `src/core/<name>.rs`: コマンド実装骨格（`assets/command-template.rs`ベース）
- `tests/cli/<name>_integration.rs`: 統合テスト骨格（`assets/test-template.rs`ベース）

**手動追加が必要:**
- `src/command.rs` の Commands enum にバリアント追加
- `src/main.rs` の match 分岐に追加

### scripts/platform-check.sh

iOS実装判断を支援します（XCUITest Runner or simctl の推奨判定）。

**使用方法:**
```bash
./scripts/platform-check.sh accessibility
# → Recommendation: Use XCUITest Runner (HTTP)

./scripts/platform-check.sh boot
# → Recommendation: Use xcrun simctl
```

### scripts/run-tests.sh

3層テスト実行を順次ガイドします。

**使用方法:**
```bash
./scripts/run-tests.sh
```

### assets/command-template.rs

コマンド実装のテンプレート（80%のユースケースに対応）。

**テンプレート変数:**
- `{COMMAND_NAME}`: コマンド名（snake_case）
- `{CommandName}`: 構造体名（PascalCase）
- `{DESCRIPTION}`: 説明文

### assets/test-template.rs

統合テストのテンプレート。

### assets/checklist.md

実装チェックリスト（各フェーズの確認項目）。

## 重要ファイル一覧

### CLI層 (src/)

| ファイル | 説明 |
|---------|------|
| `src/command.rs` | Commands enum（全コマンド定義） |
| `src/main.rs` | エントリポイント、コマンドディスパッチ |
| `src/lib.rs` | ライブラリエクスポート |
| `src/helpers/client.rs` | `with_xcuitest()` ヘルパー |
| `src/helpers/common_args.rs` | DeviceArgs, DeviceFormatArgs 等 |
| `src/helpers/format.rs` | OutputFormat (Text/Json) |
| `src/helpers/signal.rs` | Ctrl+C シグナルハンドリング |
| `src/core/*.rs` | コア操作コマンド実装 |
| `src/snapshot/` | UIスナップショット (collector, ref_generator, tree_printer) |
| `src/session/` | セッション管理 (resolver, state) |
| `src/app.rs` | アプリ管理コマンド |
| `src/device.rs` | デバイス管理コマンド |
| `src/doctor.rs` | 環境診断コマンド |

### Platform層

| ファイル | 説明 |
|---------|------|
| `crates/platform-ios/src/xcuitest/client.rs` | XCUITestClient (HTTP) |
| `crates/platform-ios/src/xcuitest/runner.rs` | Runner ライフサイクル (ensure_runner_started) |
| `crates/platform-ios/src/simctl/management.rs` | simctl 操作 (boot/shutdown等) |
| `crates/platform-ios/src/simctl/cache.rs` | DeviceCache (TTL 5秒) |
| `crates/platform-ios/src/coresim/` | CoreSimulator FFI (macOS only) |
| `crates/platform-ios/src/snapshot/extractor.rs` | extract_ios_elements() |
| `crates/platform-android/src/adb/connection.rs` | AdbConnection (TCP :5037) |
| `crates/platform-android/src/adb/commands.rs` | ADB公開API |
| `crates/platform-android/src/adb/screenshot.rs` | screencap -p (PNG→JPEG変換) |
| `crates/platform-android/src/adb/logcat.rs` | LogcatStream (ストリーミング) |
| `crates/platform-android/src/adb/permission.rs` | grant/revoke/reset permissions |
| `crates/platform-android/src/snapshot/extractor.rs` | extract_android_elements() |
| `crates/xcuitest-runner/` | Swift XCUITest Runner プロジェクト |

### Core層

| ファイル | 説明 |
|---------|------|
| `crates/core/src/lib.rs` | Platform enum, エクスポート |
| `crates/core/src/error.rs` | Error types |
| `crates/core/src/io/output.rs` | OutputWriter (stdout/file/tee) |
| `crates/core/src/snapshot/` | Frame, RawElement 共通型 |
| `crates/core/src/types/` | TargetType, Address, ScrollDirection |

### Gateway層

| ファイル | 説明 |
|---------|------|
| `crates/gateway/src/platform.rs` | DeviceResolver（プラットフォーム検出） |
| `crates/gateway/src/api/android.rs` | AndroidDevice 統一API |
| `crates/gateway/src/console.rs` | stream_console_logs（共通ログ） |

## References & Next Steps

詳細情報は以下のリファレンスファイルを参照してください:

### references/implementation-patterns.md
- `with_xcuitest()` の複数バリエーション
  - 基本形（UDIDパラメータなし）
  - 複数クライアント操作
- 引数パターン（DeviceArgs、DeviceFormatArgs、カスタム検証）
- エラーハンドリングパターン
- JSON出力パターン
- 非同期処理パターン（tokio::select!、ストリーミング応答）

### references/testing-guide.md
- 3層テスト詳細
  - ユニットテスト戦略
  - 統合テスト戦略
  - 実機確認詳細手順
- `tests/cli/common/mod.rs` ヘルパー関数リスト
- **実機確認詳細手順（/mobile-e2eスキル使用）**
- TDDサイクル実践例

### references/architecture.md
- 4層アーキテクチャ詳細（`docs/ARCHITECTURE.md`の補足）
- 各層の責務詳細
- モジュール配置規則
- データフローパターン
- Cargoワークスペース依存関係

### プロジェクト内ドキュメント

- **CLAUDE.md**: AI開発者向けクイックスタート
- **README.md**: ユーザー向け使用方法
- **crates/xcuitest-runner/ARCHITECTURE.md**: XCUITest Runner アーキテクチャ詳細

## まとめ

新機能追加時のクイックスタート:

0. **環境確認**: `cargo run -- doctor` で環境診断、または `./scripts/setup-ios.sh` / `./scripts/setup-android.sh` で環境準備
1. **判断**: `./scripts/platform-check.sh <feature>` で実装方法確認
2. **生成**: `./scripts/new-command.sh <command>` でテンプレート生成
3. **実装**: TODOコメントを埋める
4. **テスト**: `cargo build && cargo test --verbose --bins`
5. **実機確認**: `/mobile-e2e ios` or `/mobile-e2e android` で動作確認（必須!）
6. **統合テスト**: `cargo test --test cli -- --test-threads=1`
7. **コミット**: `git commit -m "feat: add <command>"`

**実機確認は必須**: ビルド成功≠動作確認

**詳細情報**: `references/` ディレクトリ参照
