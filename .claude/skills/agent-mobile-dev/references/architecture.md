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
│  - gRPC Client           │  │  - adb wrapper               │
│  - Companion管理         │  │  - Device operations         │
│  - simctl wrapper        │  │                              │
└──────────────┬───────────┘  └──────────────────────────────┘
               │
               ↓
┌──────────────────────────┐
│  idb_companion           │
│  (Swift/ObjC daemon)     │
│  - Framebuffer access    │
│  - App lifecycle         │
│  - File operations       │
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
| **iOS Platform** | tonic 0.12 + prost 0.13 | gRPC通信（idb_companion） |
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
├── idb/                    # IDB互換コマンド
│   ├── mod.rs              # IdbCommands enum
│   ├── file/               # File operations
│   ├── hid/                # Input operations
│   ├── target/             # Device management
│   ├── video/              # Video recording
│   └── ...
└── helpers/                # ヘルパー関数
    ├── client.rs           # with_client()
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
use crate::cli::helpers::{with_client, CommandResult, DeviceArgs};

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
    with_client(udid, |mut client| async move {
        // Platform層のgRPC呼び出し
        client.hid(events).await?;
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
                // 1. iOSデバイス確認（/tmp/idb/state）
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
// crates/gateway/src/api/ios.rs

pub struct IosDevice {
    client: IdbClient,
}

impl IosDevice {
    pub async fn tap(&mut self, x: f64, y: f64) -> Result<()> {
        // Platform層のgRPC呼び出し
        let events = tap_events(x, y);
        self.client.hid(events).await
    }

    pub async fn screenshot(&mut self) -> Result<Vec<u8>> {
        self.client.screenshot().await
    }

    pub async fn stream_logs(&mut self) -> Result<LogStream> {
        self.client.log(LogSource::Target, vec![]).await
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
    ├── companion/          # idb_companion管理
    │   ├── resolver.rs     # UDID → Address解決
    │   ├── state.rs        # /tmp/idb/state パース
    │   ├── spawner.rs      # companion自動起動
    │   └── lister.rs       # companion一覧
    ├── grpc/               # idb gRPC実装
    │   ├── client.rs       # IdbClient（接続管理）
    │   ├── app.rs          # アプリ操作
    │   ├── file.rs         # ファイル操作
    │   ├── hid.rs          # HID入力
    │   ├── media.rs        # スクリーンショット/動画
    │   └── ...
    ├── simctl/             # xcrun simctl wrapper
    │   └── management.rs   # boot/shutdown/create/delete
    ├── proto/              # Protocol Buffers
    │   └── idb.proto       # gRPC定義
    └── snapshot/           # UI要素抽出
        └── ...
```

**IdbClient実装**:
```rust
// crates/platform-ios/src/grpc/client.rs

pub struct IdbClient {
    inner: CompanionServiceClient<Channel>,
}

impl IdbClient {
    /// gRPC接続確立
    pub async fn connect(address: Address) -> Result<Self> {
        let channel = match address {
            Address::DomainSocket { path } => {
                // UDS接続（ローカル）
                Endpoint::try_from("http://[::]:50051")?
                    .connect_with_connector(service_fn(move |_| {
                        UnixStream::connect(path.clone())
                    }))
                    .await?
            }
            Address::Tcp { host, port } => {
                // TCP接続（リモート）
                Endpoint::from_shared(format!("http://{}:{}", host, port))?
                    .connect()
                    .await?
            }
        };

        let inner = CompanionServiceClient::new(channel);
        Ok(Self { inner })
    }

    /// HID入力
    pub async fn hid(&mut self, events: Vec<HIDEvent>) -> Result<()> {
        let stream = stream::iter(events);
        self.inner.hid(stream).await?;
        Ok(())
    }

    /// アクセシビリティ情報
    pub async fn accessibility_info(
        &mut self,
        point: Option<(f64, f64)>,
        nested: bool
    ) -> Result<String> {
        let request = AccessibilityInfoRequest { point, nested };
        let response = self.inner.accessibility_info(request).await?;
        Ok(response.into_inner().json)
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
| **IDB互換** | `src/idb/` | launch, terminate, install |
| **特殊機能** | `src/` 直下 | console, record |

### Platform実装配置

| 実装タイプ | 配置場所 | 例 |
|----------|---------|-----|
| **idb gRPC** | `crates/platform-ios/src/grpc/` | hid.rs, file.rs, media.rs |
| **xcrun simctl** | `crates/platform-ios/src/simctl/` | management.rs |
| **adb wrapper** | `crates/platform-android/src/adb/` | input.rs, uiautomator.rs |

### ヘルパー配置

| ヘルパータイプ | 配置場所 | 例 |
|-------------|---------|-----|
| **CLI共通** | `src/helpers/` | with_client(), DeviceArgs |
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
Platform Layer (IdbClient)
  │ client.hid(tap_events(100, 200))
  ↓
idb_companion (gRPC)
  │ HIDEvent stream
  ↓
iOS Simulator/Device
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
Platform Layer (IdbClient::log)
  │ LogRequest { source: Target, args: [] }
  │
  ↓ [ストリーム開始]
  │
idb_companion
  │ LogResponse stream
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
  │ client.accessibility_info(None, true)
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
│       ├── tonic
│       ├── prost
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
// ✓ Good: HID入力専用
// crates/platform-ios/src/grpc/hid.rs
impl IdbClient {
    pub async fn hid(&mut self, events: Vec<HIDEvent>) -> Result<()> {
        // HID入力のみ
    }
}

// ✗ Bad: 複数の責務
impl IdbClient {
    pub async fn do_everything(&mut self) -> Result<()> {
        // HID、ファイル、アプリ、全部やる
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

    with_client(args.device.udid.as_deref(), |mut client| async move {
        client.focus().await?;  // エラーは上位に伝播
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
- IDB互換コマンド → `src/idb/`
- idb gRPC実装 → `crates/platform-ios/src/grpc/`
- xcrun simctl → `crates/platform-ios/src/simctl/`
- adb wrapper → `crates/platform-android/src/adb/`

**関連ファイル**:
- `docs/ARCHITECTURE.md`: アーキテクチャ概要
- `CLAUDE.md`: 開発ガイド
- `Cargo.toml`: ワークスペース設定
