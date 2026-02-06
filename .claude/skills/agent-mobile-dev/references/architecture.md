# agent-mobile アーキテクチャ詳細

agent-mobile CLIの4層アーキテクチャ、モジュール配置規則、データフローパターンを詳細に解説します。

> **注**: このファイルは `docs/ARCHITECTURE.md` の補足・拡張版です。

## 目次

- [アーキテクチャ概要](#アーキテクチャ概要)
- [4層アーキテクチャ詳細](#4層アーキテクチャ詳細)
- [モジュール配置規則](#モジュール配置規則)
- [データフローパターン](#データフローパターン)
- [依存関係グラフ](#依存関係グラフ)
- [設計原則](#設計原則)

## アーキテクチャ概要

### 全体構成

```text
┌─────────────────────────────────────────────────────────────┐
│  CLI Layer (src/)                                        │
│  - コマンドパース (clap derive)                             │
│  - 引数定義のみ                                             │
│  - ビジネスロジックなし                                     │
└────────────────────────┬────────────────────────────────────┘
                         │ Gateway API呼び出し
                         ↓
┌─────────────────────────────────────────────────────────────┐
│  Gateway Layer (crates/gateway/)                             │
│  - プラットフォーム検出 (DeviceResolver)                    │
│  - 統一API (IosDevice / AndroidDevice)                      │
│  - 高レベル操作抽象化                                       │
└──────────────┬──────────────────────┬───────────────────────┘
               │                      │
               ↓ (iOS)                ↓ (Android)
┌──────────────────────────┐  ┌──────────────────────────────┐
│  iOS Platform Layer      │  │  Android Platform Layer      │
│  (crates/platform-ios/)  │  │  (crates/platform-android/)  │
│  - XCUITestClient (HTTP) │  │  - adb wrapper               │
│  - simctl wrapper        │  │  - Device operations         │
│  - XCUITest Runner       │  │                              │
└──────────────┬───────────┘  └──────────────────────────────┘
               │
               ↓
┌──────────────────────────┐
│  XCUITest Runner         │
│  (Swift, localhost:8200) │
│  - Screenshot            │
│  - Accessibility         │
│  - Clipboard             │
│  - Touch/Input           │
└──────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│  Core Layer (crates/core/)                                   │
│  - Platform enum (Ios, Android)                             │
│  - OutputWriter (stdout/file/tee)                           │
│  - 共通型・トレイト                                         │
└─────────────────────────────────────────────────────────────┘
```

### 技術スタック

| レイヤー | 技術 | 用途 |
|---------|------|------|
| **CLI** | clap 4.5 (derive) | コマンドパース、引数定義 |
| **Gateway** | Rust async/await | プラットフォーム抽象化 |
| **iOS Platform** | reqwest (HTTP) | XCUITest Runner通信 |
| **iOS Platform** | std::process::Command | xcrun simctl呼び出し |
| **Android Platform** | std::process::Command | adb呼び出し |
| **Core** | tokio 1.49 | 非同期ランタイム |

## 4層アーキテクチャ詳細

### レイヤー1: CLI Layer

**責務**:
- コマンドライン引数のパース
- 引数の型定義
- Gateway APIの呼び出し

**禁止事項**:
- ビジネスロジックの実装
- プラットフォーム固有の処理
- 直接的なデバイス操作

**ディレクトリ構造**:
```text
src/
├── mod.rs                  # CLI entry point, Commands enum
├── core/                   # トップレベルコマンド
│   ├── tap.rs
│   ├── swipe.rs
│   ├── scroll.rs
│   ├── long_press.rs
│   ├── type_cmd.rs
│   ├── fill.rs
│   ├── find.rs
│   ├── get.rs
│   ├── is_cmd.rs
│   ├── wait.rs
│   └── screenshot.rs
├── console.rs              # Console log streaming
├── record.rs               # Screen recording
└── helpers/                # ヘルパー関数
    ├── client.rs           # with_xcuitest()
    ├── common_args.rs      # DeviceArgs, DeviceFormatArgs
    ├── format.rs           # OutputFormat
    └── ...
```

**実装パターン**:
```rust
// src/core/tap.rs

use clap::Args;
use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;
use crate::cli::helpers::{with_xcuitest, CommandResult, DeviceArgs};

#[derive(Args, Debug)]
pub struct TapArgs {
    pub target: String,

    #[command(flatten)]
    pub device: DeviceArgs,
}

pub async fn run(args: TapArgs) -> CommandResult {
    // 1. プラットフォーム検出（Gateway API）
    let platform = DeviceResolver::resolve_platform(
        args.device.platform.as_deref()
    ).await?;

    // 2. プラットフォーム別処理（Gateway API）
    match platform {
        Platform::Ios => run_ios(args.device.udid.as_deref()).await,
        Platform::Android => run_android(args.device.udid.as_deref()).await,
    }
}

// 3. iOS実装（Gateway/Platform API使用）
async fn run_ios(udid: Option<&str>) -> CommandResult {
    with_xcuitest(udid, |client| async move {
        // Platform層のXCUITest Runner HTTP呼び出し
        client.tap(x, y).await?;
        Ok(())
    }).await
}
```

### レイヤー2: Gateway Layer

**責務**:
- プラットフォーム自動検出
- 統一されたデバイスAPI提供
- 高レベル操作の抽象化

**提供API**:
- `DeviceResolver::resolve_platform()`: プラットフォーム検出
- `IosDevice`: iOS統一API
- `AndroidDevice`: Android統一API
- `stream_console_logs()`: コンソールログストリーミング

**ディレクトリ構造**:
```text
crates/gateway/
└── src/
    ├── lib.rs              # Gateway entry point
    ├── platform.rs         # DeviceResolver (プラットフォーム検出)
    ├── console.rs          # Console streaming (iOS/Android)
    ├── api/
    │   ├── ios.rs          # IosDevice API
    │   └── android.rs      # AndroidDevice API (WIP)
    └── ...
```

**DeviceResolver実装**:
```rust
// crates/gateway/src/platform.rs

pub struct DeviceResolver;

impl DeviceResolver {
    /// プラットフォーム自動検出
    pub async fn resolve_platform(
        platform_hint: Option<&str>
    ) -> Result<Platform> {
        match platform_hint {
            Some("ios") => Ok(Platform::Ios),
            Some("android") => Ok(Platform::Android),
            None => {
                // 1. iOSデバイス確認（simctl list_simulators()）
                if Self::has_ios_devices().await? {
                    return Ok(Platform::Ios);
                }
                // 2. Androidデバイス確認（adb devices）
                if Self::has_android_devices().await? {
                    return Ok(Platform::Android);
                }
                Err("No devices found".into())
            }
            Some(p) => Err(format!("Unknown platform: {}", p).into()),
        }
    }
}
```

**IosDevice API**:
```rust
// crates/platform-ios/src/xcuitest/client.rs

pub struct XCUITestClient {
    base_url: String,  // http://localhost:8200
}

impl XCUITestClient {
    pub async fn tap(&self, x: f64, y: f64) -> Result<()> {
        // XCUITest Runner HTTP呼び出し
        self.post("/tap", json!({ "x": x, "y": y })).await
    }

    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        self.get_bytes("/screenshot").await
    }

    pub async fn accessibility_info(&self) -> Result<String> {
        self.get("/accessibility").await
    }
}
```

### レイヤー3: Platform Layer

**責務**:
- プラットフォーム固有の実装
- 外部プロセスとの通信
- 低レベルAPI提供

**iOS Platform Layer**:

```text
crates/platform-ios/
└── src/
    ├── lib.rs
    ├── xcuitest/           # XCUITest Runner通信
    │   └── client.rs       # XCUITestClient（HTTP, localhost:8200）
    ├── simctl/             # xcrun simctl wrapper
    │   ├── management.rs   # boot/shutdown/install/uninstall/list_simulators
    │   └── cache.rs        # DeviceCache (TTL 5秒)
    └── snapshot/           # UI要素抽出
        └── ...
```

**XCUITestClient実装**:
```rust
// crates/platform-ios/src/xcuitest/client.rs

pub struct XCUITestClient {
    base_url: String,  // http://localhost:8200
    client: reqwest::Client,
}

impl XCUITestClient {
    /// HTTP接続確立
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://localhost:{}", port),
            client: reqwest::Client::new(),
        }
    }

    /// タップ操作
    pub async fn tap(&self, x: f64, y: f64) -> Result<()> {
        self.client.post(&format!("{}/tap", self.base_url))
            .json(&json!({ "x": x, "y": y }))
            .send().await?;
        Ok(())
    }

    /// アクセシビリティ情報
    pub async fn accessibility_info(&self) -> Result<String> {
        let response = self.client.get(&format!("{}/accessibility", self.base_url))
            .send().await?;
        Ok(response.text().await?)
    }
}
```

**simctl wrapper実装**:
```rust
// crates/platform-ios/src/simctl/management.rs

use std::process::Command;

pub fn boot(udid: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "boot", udid])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to boot: {}", stderr).into());
    }

    Ok(())
}

pub fn shutdown(udid: &str) -> Result<()> {
    let output = Command::new("xcrun")
        .args(["simctl", "shutdown", udid])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to shutdown: {}", stderr).into());
    }

    Ok(())
}
```

**Android Platform Layer**:

```text
crates/platform-android/
└── src/
    ├── lib.rs
    └── adb/                # adb wrapper
        ├── input.rs        # Input operations (tap, swipe, keyevent)
        ├── uiautomator.rs  # UI Automator (dump, parse)
        └── ...
```

### レイヤー4: Core Layer

**責務**:
- 共通型・トレイト定義
- 汎用ユーティリティ
- プラットフォーム非依存の機能

**ディレクトリ構造**:
```text
crates/core/
└── src/
    ├── lib.rs              # Core entry point, Platform enum
    └── io/
        └── output.rs       # OutputWriter (stdout/file/tee)
```

**Platform enum**:
```rust
// crates/core/src/lib.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Ios,
    Android,
}
```

**OutputWriter**:
```rust
// crates/core/src/io/output.rs

pub enum OutputWriter {
    Stdout(io::Stdout),           // stdout only
    File(File),                   // file only
    Tee { file: File, stdout: io::Stdout },  // both
}

impl OutputWriter {
    pub fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match self {
            Self::Stdout(stdout) => stdout.write_all(buf),
            Self::File(file) => file.write_all(buf),
            Self::Tee { file, stdout } => {
                file.write_all(buf)?;
                stdout.write_all(buf)
            }
        }
    }
}
```

## モジュール配置規則

### コマンド配置

| コマンドタイプ | 配置場所 | 例 |
|-------------|---------|-----|
| **トップレベル（AI最適化）** | `src/core/` | tap, swipe, find, screenshot |
| **特殊機能** | `src/` 直下 | console, record |

### Platform実装配置

| 実装タイプ | 配置場所 | 例 |
|----------|---------|-----|
| **XCUITest Runner** | `crates/platform-ios/src/xcuitest/` | client.rs |
| **xcrun simctl** | `crates/platform-ios/src/simctl/` | management.rs, cache.rs |
| **adb wrapper** | `crates/platform-android/src/adb/` | input.rs, uiautomator.rs |

### ヘルパー配置

| ヘルパータイプ | 配置場所 | 例 |
|-------------|---------|-----|
| **CLI共通** | `src/helpers/` | with_xcuitest(), DeviceArgs |
| **テスト** | `tests/cli/common/` | assert_success(), run_cli_command() |

## データフローパターン

### パターン1: 単純なコマンド実行

```text
ユーザー
  │
  │ agent-mobile tap 100,200
  ↓
CLI Layer (src/core/tap.rs)
  │ TapArgs { target: "100,200", device: DeviceArgs }
  ↓
Gateway Layer (DeviceResolver)
  │ resolve_platform() → Platform::Ios
  ↓
CLI Layer (run_ios)
  │ with_client(udid, |client| {...})
  ↓
Platform Layer (XCUITestClient)
  │ client.tap(100, 200) via HTTP
  ↓
XCUITest Runner (localhost:8200)
  │ XCUITest API call
  ↓
iOS Simulator
  │ タップ実行
  ↓
ユーザー（視覚的フィードバック）
```

### パターン2: ストリーミング

```text
ユーザー
  │
  │ agent-mobile console
  ↓
CLI Layer (src/console.rs)
  │ ConsoleArgs { device: DeviceArgs, output: None }
  ↓
Gateway Layer
  │ stream_console_logs(platform, udid, writer, stop_rx)
  ↓
Platform Layer (simctl spawn log)
  │ os_log stream
  │
  ↓ [ストリーム開始]
  │
simctl process
  │ log output stream
  ↓
Gateway Layer
  │ ログエントリ処理
  ↓
Core Layer (OutputWriter)
  │ write_all(log_line)
  ↓
ユーザー（stdout/file）
  │
  │ [Ctrl+C]
  ↓
CLI Layer
  │ stop_tx.send(true)
  ↓
Gateway Layer
  │ select! { _ = stop_rx.changed() => break }
  ↓
[ストリーム終了]
```

### パターン3: JSON出力

```text
ユーザー
  │
  │ agent-mobile find "Login" --format json
  ↓
CLI Layer (src/core/find.rs)
  │ FindArgs { query: "Login", device: DeviceFormatArgs }
  ↓
Gateway Layer
  │ resolve_platform() → Platform::Ios
  ↓
Platform Layer
  │ client.accessibility_info()
  ↓
CLI Layer
  │ JSON解析、要素抽出
  │
  ├─ OutputFormat::Json
  │  │ serde_json::json!({ "elements": [...] })
  │  ↓ println!("{}", json)
  │
  └─ OutputFormat::Human
     │ "Found 3 elements:"
     ↓ println!("  @e1 - Login button")
```

## 依存関係グラフ

### Cargoワークスペース構造

```text
agent-mobile (root)
│
├── src/                    # Binary crate
│   └── [depends on]
│       ├── agent_mobile_core
│       ├── agent_mobile_gateway
│       ├── agent_mobile_platform_ios
│       └── agent_mobile_platform_android
│
├── crates/core/            # 共通型
│   └── [depends on] なし
│
├── crates/gateway/         # プラットフォーム抽象化
│   └── [depends on]
│       ├── agent_mobile_core
│       ├── agent_mobile_platform_ios
│       └── agent_mobile_platform_android
│
├── crates/platform-ios/    # iOS実装
│   └── [depends on]
│       ├── agent_mobile_core
│       ├── reqwest
│       └── tokio
│
└── crates/platform-android/ # Android実装
    └── [depends on]
        ├── agent_mobile_core
        └── tokio
```

### 依存関係の方向

```text
[上位レイヤー] → [下位レイヤー]

CLI → Gateway → Platform → Core
 │        │         │
 │        │         └─ Core
 │        └─ Platform
 └─ Gateway
```

**原則**: 下位レイヤーは上位レイヤーに依存しない（単方向依存）。

## 設計原則

### 1. 関心の分離（Separation of Concerns）

- **CLI層**: コマンドパースのみ
- **Gateway層**: プラットフォーム抽象化のみ
- **Platform層**: プラットフォーム固有実装のみ
- **Core層**: 共通機能のみ

### 2. 依存性逆転（Dependency Inversion）

```rust
// ✓ Good: Gateway層がPlatform層のトレイトに依存
pub trait DeviceOperations {
    async fn tap(&mut self, x: f64, y: f64) -> Result<()>;
}

impl DeviceOperations for IosDevice {
    async fn tap(&mut self, x: f64, y: f64) -> Result<()> {
        // iOS実装
    }
}

// ✗ Bad: Gateway層がPlatform層の具象型に依存
pub fn tap_device(ios_device: IosDevice) {
    // Gateway層がPlatform層の詳細に依存
}
```

### 3. 単一責任原則（Single Responsibility）

各モジュールは1つの責務のみを持つ:

```rust
// ✓ Good: タップ操作専用
// crates/platform-ios/src/xcuitest/client.rs
impl XCUITestClient {
    pub async fn tap(&self, x: f64, y: f64) -> Result<()> {
        // タップ操作のみ
    }
}

// ✗ Bad: 複数の責務
impl XCUITestClient {
    pub async fn do_everything(&self) -> Result<()> {
        // タップ、スクリーンショット、アクセシビリティ、全部やる
    }
}
```

### 4. 開放閉鎖原則（Open/Closed）

拡張に開かれ、修正に閉じている:

```rust
// ✓ Good: 新プラットフォーム追加時、既存コード変更不要
pub enum Platform {
    Ios,
    Android,
    // 将来: Windows, macOS, etc.
}

// ✗ Bad: 新プラットフォーム追加時、全コマンドを修正
pub fn run(args: MyArgs) -> Result<()> {
    if is_ios() {
        // iOS処理
    } else if is_android() {
        // Android処理
    }
    // 新プラットフォーム追加時、全てのif文を修正
}
```

### 5. エラー伝播の一貫性

```rust
// ✓ Good: ?演算子でエラー伝播
pub async fn run(args: MyArgs) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(
        args.device.platform.as_deref()
    ).await?;  // エラーは上位に伝播

    with_xcuitest(args.device.udid.as_deref(), |client| async move {
        client.tap(100.0, 200.0).await?;  // エラーは上位に伝播
        Ok(())
    }).await
}

// ✗ Bad: エラーを握りつぶす
pub async fn run(args: MyArgs) -> CommandResult {
    match DeviceResolver::resolve_platform(...).await {
        Ok(platform) => { /* ... */ }
        Err(_) => { /* エラーを握りつぶす */ }
    }
    Ok(())  // 常に成功を返す（問題）
}
```

## まとめ

**アーキテクチャの要点**:
1. **4層分離**: CLI → Gateway → Platform → Core
2. **単方向依存**: 上位→下位、下位は上位に依存しない
3. **責務分離**: 各層は明確な責務のみ
4. **プラットフォーム抽象化**: Gateway層で統一API
5. **エラー伝播**: ?演算子で一貫したエラーハンドリング

**配置規則**:
- トップレベルコマンド → `src/core/`
- XCUITest Runner実装 → `crates/platform-ios/src/xcuitest/`
- xcrun simctl → `crates/platform-ios/src/simctl/`
- adb wrapper → `crates/platform-android/src/adb/`

**関連ファイル**:
- `docs/ARCHITECTURE.md`: アーキテクチャ概要
- `CLAUDE.md`: 開発ガイド
- `Cargo.toml`: ワークスペース設定
