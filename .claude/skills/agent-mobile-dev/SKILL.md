---
name: agent-mobile-dev
description: Guide for developing CLI commands in agent-mobile (Rust mobile E2E testing tool). Use when adding new commands, implementing features, or refactoring agent-mobile code. Enforces 4-layer architecture (CLI/Gateway/Platform/Core), platform decision-making (iOS: XCUITest Runner (HTTP) vs xcrun simctl), and mandatory real-device testing workflow (Phase 0 environment setup → design → implement → test → device verification → commit).
version: 2.0.0
argument-hint: "[setup-ios|setup-android|<other-args>]"
---

# agent-mobile CLI開発スキル

agent-mobile CLI（Rust製モバイルE2Eテストツール）の新機能開発、既存機能リファクタリングのための包括的ガイド。4層アーキテクチャの理解、プラットフォーム実装判断、必須開発フローを体系化しています。

## 概要

### agent-mobileとは

**agent-mobile**は、iOS/Android両対応のRust製モバイル自動化CLIツールです。AI開発者が効率的に使えるように設計されています。

**主要特徴:**
- **4層アーキテクチャ**: CLI → Gateway → Platform → Core
- **iOS対応**: XCUITest Runner (HTTP) + xcrun simctl ハイブリッド
- **Android対応**: adb wrapper
- **AI最適化**: 簡潔なコマンド、JSON出力、エラーメッセージ明確化

**技術スタック:**
```
Rust 2021 | tokio 1.49 | reqwest (HTTP) | clap 4.5
```

### 4層アーキテクチャ

```text
┌─────────────────────────────────────────┐
│  CLI Layer (src/)                   │
│  - コマンドパース (clap derive)        │
│  - 引数定義のみ                         │
│  - Gateway API呼び出し                  │
└───────────────┬─────────────────────────┘
                │
                ↓
┌─────────────────────────────────────────┐
│  Gateway Layer (crates/gateway/)        │
│  - プラットフォーム検出                 │
│  - 統一API (IosDevice/AndroidDevice)   │
│  - 高レベル操作                         │
└────┬──────────────────────┬─────────────┘
     │                      │
     ↓ (iOS)                ↓ (Android)
┌──────────────┐      ┌─────────────────┐
│ iOS Platform │      │ Android Platform│
│ - XCUITest   │      │ - adb wrapper   │
│ - simctl     │      │                 │
└──────────────┘      └─────────────────┘
┌─────────────────────────────────────────┐
│  Core Layer (crates/core/)              │
│  - Platform enum                        │
│  - OutputWriter (stdout/file/tee)      │
└─────────────────────────────────────────┘
```

**各層の責務:**
- **CLI層**: コマンドパース、引数検証のみ
- **Gateway層**: プラットフォーム抽象化、統一API提供
- **Platform層**: iOS (XCUITest Runner HTTP/simctl)、Android (adb) 実装
- **Core層**: 共通型、OutputWriter

## Quick Start: 新機能追加の判断フロー

新しいCLIコマンドを追加する際の判断フローチャート:

```text
1. 機能の分類
   ├─ トップレベルコマンド？ (tap, swipe, find...)
   │  └─ src/core/<name>.rs に実装
   └─ サブコマンド？
      └─ 適切なモジュールに配置

2. iOS実装判断
   ├─ XCUITest Runner (HTTP)で実装可能？
   │  ├─ Yes → XCUITest Runner (HTTP) で実装
   │  │  └─ with_xcuitest() パターン使用
   │  └─ No → xcrun simctl で実装
   │     └─ simctl::management モジュール拡張
   └─ 両方で可能？
      └─ ハイブリッド (XCUITest Runner優先、fallback)

3. Android実装判断
   ├─ adb コマンドで実装可能？
   │  └─ crates/platform-android/src/adb/ に実装
   └─ 複雑な操作？
      └─ Gateway層でラッパー実装

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
2. Android: adbコマンドで実現可能か確認

**引数設計:**
```rust
#[derive(Args, Debug)]
pub struct MyCommandArgs {
    /// Target: @eN ref, "text", or coordinates
    pub target: String,

    #[command(flatten)]
    pub device: DeviceArgs,  // --udid, --platform
}
```

### ステップ2: 実装

**コマンドテンプレート使用:**
```bash
./scripts/new-command.sh vibrate
# 生成:
# ✓ src/core/vibrate.rs
# ✓ tests/cli/vibrate_integration.rs
# ✓ src/mod.rs に自動追記
```

**手動実装の場合:**
```rust
// src/core/my_feature.rs
use clap::Args;
use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;
use crate::cli::helpers::{with_xcuitest, CommandResult, DeviceArgs};

#[derive(Args, Debug)]
pub struct MyFeatureArgs {
    #[command(flatten)]
    pub device: DeviceArgs,
}

pub async fn run(args: MyFeatureArgs) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(
        args.device.platform.as_deref()
    ).await?;

    match platform {
        Platform::Ios => run_ios(args.device.udid.as_deref()).await,
        Platform::Android => run_android(args.device.udid.as_deref()).await,
    }
}

async fn run_ios(udid: Option<&str>) -> CommandResult {
    with_xcuitest(udid, |mut client| async move {
        // XCUITest Runner (HTTP)実装
        Ok(())
    }).await
}

async fn run_android(udid: Option<&str>) -> CommandResult {
    // adb実装
    Ok(())
}
```

**src/mod.rsに追加:**
```rust
// mod宣言
pub mod my_feature;

// Commands enum
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ...
    MyFeature(my_feature::MyFeatureArgs),
}

// match分岐
Commands::MyFeature(args) => my_feature::run(args).await,
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

**基本形:**
```rust
pub async fn run(udid: Option<String>) -> CommandResult {
    with_xcuitest(udid.as_deref(), |mut client| async move {
        // XCUITest Runner (HTTP)操作
        client.accessibility_info(None, true).await?;
        Ok(())
    }).await
}
```

**ストリーミング用:**
```rust
use crate::cli::helpers::with_xcuitest_streaming;

pub async fn run(udid: Option<String>) -> CommandResult {
    with_xcuitest_streaming(udid.as_deref(), |mut client| async move {
        let mut stream = client.log(LogSource::Target, vec![]).await?;
        // ストリーム処理...
        Ok(())
    }).await
}
```

### DeviceArgs flatten パターン

デバイス指定引数（--udid、--platform）を標準化します。

```rust
#[derive(Args, Debug)]
pub struct MyCommandArgs {
    /// コマンド固有の引数
    pub target: String,

    #[command(flatten)]
    pub device: DeviceArgs,  // --udid, --platform を自動追加
}
```

### エラーハンドリング

**CommandResult型:**
```rust
pub type CommandResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
```

**使用例:**
```rust
pub async fn run(args: MyArgs) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref())
        .await?;  // ?演算子でエラー伝播

    if platform == Platform::Ios {
        with_xcuitest(args.device.udid.as_deref(), |mut client| async move {
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
    pub device: DeviceFormatArgs,  // --udid, --platform, --format
}

pub async fn run(args: MyCommandArgs) -> CommandResult {
    let result = perform_operation().await?;

    match args.device.format.format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "status": "success",
                "data": result
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Human => {
            println!("Result: {}", result);
        }
    }
    Ok(())
}
```

## プラットフォーム判断（iOS）

### 判断フローチャート

```text
XCUITest Runner (HTTP) で実装可能？
├─ Yes → XCUITest Runner (HTTP) 実装
│  └─ with_xcuitest() 使用
├─ No  → xcrun simctl 実装
│  └─ simctl::management 拡張
└─ 両方？→ ハイブリッド
   └─ XCUITest Runner優先、fallback で simctl
```

### 機能別推奨実装

| 機能カテゴリ | 推奨実装 | 理由 |
|------------|---------|------|
| HID入力 (tap/swipe/text) | **XCUITest Runner (HTTP)** | 精密な制御、ストリーミング対応 |
| アクセシビリティ | **XCUITest Runner (HTTP)** | simctlでは不可 |
| スクリーンショット | **XCUITest Runner (HTTP)** | Framebuffer直接アクセス |
| アプリ操作 (launch/terminate) | **XCUITest Runner (HTTP)** | 進捗ストリーミング |
| ファイル操作 | **XCUITest Runner (HTTP)** | 統一インターフェース |
| シミュレータ起動/停止 | **xcrun simctl** | デバイスライフサイクル管理 |
| シミュレータ作成/削除 | **xcrun simctl** | デバイスライフサイクル管理 |
| クリップボード | **XCUITest Runner (HTTP)** | UIPasteboard.general 経由 |

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

**詳細**: `references/platform-decisions.md` 参照

## スクリプト & テンプレート

### scripts/new-command.sh

新しいCLIコマンドの骨格を自動生成します。

**使用方法:**
```bash
./scripts/new-command.sh vibrate

# 対話:
# > Description (optional): Vibrate device
#
# 生成:
# ✓ Created src/core/vibrate.rs
# ✓ Created tests/cli/vibrate_integration.rs
# ✓ Updated src/mod.rs (added VibrateArgs, vibrate command)
#
# Next steps:
# 1. cargo build
# 2. Customize vibrate.rs implementation
# 3. cargo test --test cli vibrate
```

**生成内容:**
- `src/core/<name>.rs`: コマンド実装骨格（`assets/command-template.rs`ベース）
- `tests/cli/<name>_integration.rs`: 統合テスト骨格（`assets/test-template.rs`ベース）
- `src/mod.rs`: 自動的にmod宣言、Commands enum、match分岐を追加

### scripts/platform-check.sh

iOS実装判断を支援します（XCUITest Runner or simctl の推奨判定）。

**使用方法:**
```bash
./scripts/platform-check.sh accessibility

# 出力:
# Recommendation: Use XCUITest Runner (HTTP)
# Implementation: with_xcuitest() pattern
# API: accessibility_info()
```

```bash
./scripts/platform-check.sh clipboard

# 出力:
# Recommendation: Use XCUITest Runner (HTTP)
# Implementation: UIPasteboard.general 経由
```

### scripts/run-tests.sh

3層テスト実行を順次ガイドします。

**使用方法:**
```bash
./scripts/run-tests.sh

# 出力:
# Step 1: Unit tests
# $ cargo test --verbose --bins
#
# Step 2: Integration tests
# $ cargo test --test cli -- --test-threads=1
#
# Step 3: Real device verification (REQUIRED!)
# $ /mobile-e2e ios
# or
# $ /mobile-e2e android
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

## References & Next Steps

詳細情報は以下のリファレンスファイルを参照してください:

### references/implementation-patterns.md
- `with_xcuitest()` の複数バリエーション
  - 基本形
  - ストリーミング用
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
- **docs/ARCHITECTURE.md**: アーキテクチャ詳細
- **crates/xcuitest-runner/**: XCUITest Runner (Swift) プロジェクト

## まとめ

新機能追加時のクイックスタート:

0. **環境確認**: `./scripts/setup-ios.sh` または `./scripts/setup-android.sh` で環境準備
1. **判断**: `./scripts/platform-check.sh <feature>` で実装方法確認
2. **生成**: `./scripts/new-command.sh <command>` でテンプレート生成
3. **実装**: TODOコメントを埋める
4. **テスト**: `cargo build && cargo test --verbose --bins`
5. **実機確認**: `/mobile-e2e ios` or `/mobile-e2e android` で動作確認（必須!）
6. **統合テスト**: `cargo test --test cli -- --test-threads=1`
7. **コミット**: `git commit -m "feat: add <command>"`

**実機確認は必須**: ビルド成功≠動作確認

**詳細情報**: `references/` ディレクトリ参照
