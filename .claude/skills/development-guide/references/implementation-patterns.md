# agent-mobile 実装パターン集

agent-mobile CLI開発における頻出パターンとベストプラクティスを詳細に解説します。

## 目次

- [with_xcuitest() パターン](#with_xcuitest-パターン)
- [引数パターン](#引数パターン)
- [エラーハンドリング](#エラーハンドリング)
- [JSON出力パターン](#json出力パターン)
- [非同期処理パターン](#非同期処理パターン)
- [プラットフォーム分岐パターン](#プラットフォーム分岐パターン)
- [ストリーミングパターン](#ストリーミングパターン)

## with_xcuitest() パターン

`with_xcuitest()` は XCUITest Runner への接続を抽象化するヘルパー関数です。

### 基本形

**最も頻繁に使用するパターン**:

```rust
use crate::helpers::client::{with_xcuitest, CommandResult};
use crate::helpers::common_args::DeviceArgs;

pub async fn run(args: MyArgs) -> CommandResult {
    with_xcuitest(|client| async move {
        // XCUITest Runner HTTP操作
        client.tap(100.0, 200.0).await?;
        Ok(())
    }).await
}
```

**処理フロー**:
1. XCUITest Runnerの自動起動（未起動の場合）
2. XCUITestClient (HTTP, localhost:8200) を作成
3. クロージャ内でHTTP操作
4. `with_xcuitest()` はUDIDパラメータを取らない（プラットフォーム分岐はCLI層で行う）

### 長時間操作用

**アクセシビリティ走査など長時間かかる操作の場合**:

```rust
use crate::helpers::client::CommandResult;

pub async fn run(args: MyArgs) -> CommandResult {
    with_xcuitest(|client| async move {
        // accessibility_info はWebView含むアプリで60秒以上かかる場合あり
        // XCUITestClient のタイムアウトは120秒に設定
        let json_str = client.accessibility_info(None, true).await?;
        println!("{}", json_str);
        Ok(())
    }).await
}
```

**注意**: `with_xcuitest()` はRunner自動起動・ヘルスチェックを内部で行います。UDIDパラメータは不要です。

### 複数HTTP操作

```rust
pub async fn run(args: MyArgs) -> CommandResult {
    with_xcuitest(|mut client| async move {
        // 1. アクセシビリティ情報取得
        let json_str = client.accessibility_info(None, true).await?;
        let json: serde_json::Value = serde_json::from_str(&json_str)?;

        // 2. 要素を検索
        let element = find_element(&json, "Login")?;

        // 3. タップ
        let events = tap_events(element.x, element.y);
        client.hid(events).await?;

        Ok(())
    }).await
}
```

### 戻り値がある場合

```rust
pub async fn get_screen_size() -> CommandResult<(u32, u32)> {
    with_xcuitest(|client| async move {
        let desc = client.describe(false).await?;
        let dims = desc.screen_dimensions;
        Ok((dims.width as u32, dims.height as u32))
    }).await
}
```

### エラーハンドリング付き

```rust
pub async fn run(args: MyArgs) -> CommandResult {
    with_xcuitest(|mut client| async move {
        match client.accessibility_info(None, true).await {
            Ok(json_str) => {
                process_json(&json_str)?;
                Ok(())
            }
            Err(e) if is_framebuffer_error(&e) => {
                Err("Framebuffer not ready. Try waiting a few seconds after boot.".into())
            }
            Err(e) => Err(e),
        }
    }).await
}
```

## 引数パターン

### DeviceArgs flatten

**標準パターン** (全コマンドで使用):

```rust
use clap::Args;
use crate::helpers::common_args::DeviceArgs;

#[derive(Args, Debug)]
pub struct MyCommandArgs {
    /// コマンド固有の引数
    pub target: String,

    #[command(flatten)]
    pub device: DeviceArgs,  // --udid を追加（platformは自動検出）
}
```

**生成される引数**:
- `--udid <UDID>`: デバイスUDID/serial（省略時は最初の利用可能デバイスを自動検出）
- プラットフォームはUDIDから自動検出（`DeviceResolver::detect_platform()`）

### DeviceFormatArgs

**JSON出力対応コマンド用**:

```rust
use crate::helpers::common_args::DeviceFormatArgs;

#[derive(Args, Debug)]
pub struct MyCommandArgs {
    pub target: String,

    #[command(flatten)]
    pub device: DeviceFormatArgs,  // --udid, --format を追加
}
```

**追加される引数**:
- `--format <json|human>`: 出力形式（デフォルト: human）

### カスタム引数検証

```rust
use clap::Args;

#[derive(Args, Debug)]
pub struct TapArgs {
    /// Target: @eN ref, "text", or x,y coordinates
    pub target: String,

    /// Duration of tap in seconds
    #[arg(long, value_parser = parse_duration)]
    pub duration: Option<f64>,

    #[command(flatten)]
    pub device: DeviceArgs,
}

fn parse_duration(s: &str) -> Result<f64, String> {
    let duration: f64 = s.parse().map_err(|_| "Invalid duration")?;
    if duration < 0.0 || duration > 10.0 {
        return Err("Duration must be between 0 and 10 seconds".into());
    }
    Ok(duration)
}
```

### オプショナル引数

```rust
#[derive(Args, Debug)]
pub struct ScrollArgs {
    /// Direction: up, down, left, right
    pub direction: String,

    /// Element to scroll within (default: whole screen)
    #[arg(long)]
    pub in_element: Option<String>,

    /// Distance in pixels
    #[arg(long, default_value = "100")]
    pub distance: u32,

    #[command(flatten)]
    pub device: DeviceArgs,
}
```

### 複数値引数

```rust
#[derive(Args, Debug)]
pub struct LaunchArgs {
    /// Bundle ID
    pub bundle_id: String,

    /// Environment variables (KEY=VALUE)
    #[arg(long = "env", value_name = "KEY=VALUE")]
    pub env: Vec<String>,

    /// App arguments
    #[arg(last = true)]
    pub app_args: Vec<String>,

    #[command(flatten)]
    pub device: DeviceArgs,
}
```

## エラーハンドリング

### CommandResult型

```rust
pub type CommandResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
```

**特徴**:
- デフォルト型は `()`（戻り値なし）
- `?` 演算子でエラー伝播
- `Send + Sync` でスレッド安全

### 基本的なエラー伝播

```rust
pub async fn run(args: MyArgs) -> CommandResult {
    let platform = DeviceResolver::detect_platform()
        .await?;  // エラーはそのまま伝播

    with_xcuitest(|mut client| async move {
        client.focus().await?;  // エラーはそのまま伝播
        Ok(())
    }).await
}
```

### カスタムエラーメッセージ

```rust
pub async fn run(args: MyArgs) -> CommandResult {
    let platform = DeviceResolver::resolve_platform(args.device.platform.as_deref())
        .await
        .map_err(|e| format!("Failed to detect platform: {}. Ensure a device is connected.", e))?;

    Ok(())
}
```

### 条件付きエラー

```rust
pub async fn run(args: TapArgs) -> CommandResult {
    if args.target.is_empty() {
        return Err("Target cannot be empty. Use @eN, \"text\", or x,y coordinates.".into());
    }

    // 実装
    Ok(())
}
```

### エラー種別に応じた処理

```rust
pub async fn run(args: MyArgs) -> CommandResult {
    match with_xcuitest(|mut client| async move {
        client.accessibility_info(None, true).await
    }).await {
        Ok(json_str) => {
            process_json(&json_str)?;
            Ok(())
        }
        Err(e) if is_connection_error(&e) => {
            Err("Failed to connect to XCUITest Runner. Ensure the simulator is running and XCUITest Runner is started.".into())
        }
        Err(e) if is_framebuffer_error(&e) => {
            Err("Framebuffer not ready. Wait a few seconds after boot and retry.".into())
        }
        Err(e) => Err(format!("Unexpected error: {}", e).into()),
    }
}
```

### アクション可能なエラーメッセージ

**悪い例**:
```rust
Err("Error".into())
Err("Failed".into())
```

**良い例**:
```rust
Err("Device not found. Run 'agent-mobile device list' to see available devices.".into())
Err("Screenshot failed: Framebuffer not ready. Wait 5 seconds after boot and retry.".into())
Err("Invalid element ref '@e999'. Run 'agent-mobile find <text>' to find elements.".into())
```

**エラーメッセージの原則**:
1. **何が問題か**: 明確に状況を説明
2. **なぜ失敗したか**: 原因を示す（可能な場合）
3. **どう解決するか**: 次のアクションを提示

## JSON出力パターン

### 基本形

```rust
use crate::helpers::{DeviceFormatArgs, OutputFormat};
use serde_json::json;

pub async fn run(args: MyArgs) -> CommandResult {
    let result = perform_operation(&args).await?;

    match args.device.format {
        OutputFormat::Json => {
            let json = json!({
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

### 構造化データの出力

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ElementInfo {
    ref_id: String,
    label: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

pub async fn run(args: FindArgs) -> CommandResult {
    let elements = find_elements(&args.query).await?;

    match args.device.format {
        OutputFormat::Json => {
            let json = json!({
                "query": args.query,
                "count": elements.len(),
                "elements": elements
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        OutputFormat::Text => {
            println!("Found {} elements:", elements.len());
            for elem in elements {
                println!("  {} - {} ({}, {})", elem.ref_id, elem.label, elem.x, elem.y);
            }
        }
    }

    Ok(())
}
```

### エラー時のJSON出力

```rust
pub async fn run(args: MyArgs) -> CommandResult {
    let result = perform_operation(&args).await;

    match args.device.format {
        OutputFormat::Json => {
            match result {
                Ok(data) => {
                    let json = json!({
                        "status": "success",
                        "data": data
                    });
                    println!("{}", serde_json::to_string_pretty(&json)?);
                    Ok(())
                }
                Err(e) => {
                    let json = json!({
                        "status": "error",
                        "error": e.to_string()
                    });
                    eprintln!("{}", serde_json::to_string_pretty(&json)?);
                    Err(e)
                }
            }
        }
        OutputFormat::Text => result,
    }
}
```

## 非同期処理パターン

### tokio::select! でシグナルハンドリング

```rust
use tokio::select;
use tokio::sync::watch;
use crate::helpers::setup_ctrl_c_handler;

pub async fn run(args: MyArgs) -> CommandResult {
    let (stop_tx, mut stop_rx) = watch::channel(false);

    // Ctrl+Cハンドラー設定
    setup_ctrl_c_handler(stop_tx.clone());

    with_xcuitest(|mut client| async move {
        let mut stream = client.log(LogSource::Target, vec![]).await?;

        loop {
            select! {
                // ログエントリ受信
                log_result = stream.message() => {
                    match log_result {
                        Ok(Some(entry)) => println!("{}", entry),
                        Ok(None) => break,  // ストリーム終了
                        Err(e) => return Err(e.into()),
                    }
                }
                // Ctrl+C受信
                _ = stop_rx.changed() => {
                    if *stop_rx.borrow() {
                        println!("\nStopping...");
                        break;
                    }
                }
            }
        }

        Ok(())
    }).await
}
```

### ストリーミング応答処理

```rust
pub async fn run(args: InstallArgs) -> CommandResult {
    with_xcuitest(|mut client| async move {
        let mut stream = client.install(&args.path).await?;

        while let Some(response) = stream.message().await? {
            match response.status {
                Status::Progress(p) => {
                    print!("\rInstalling... {}%", p.percentage);
                    std::io::stdout().flush()?;
                }
                Status::Complete => {
                    println!("\nInstallation complete");
                    break;
                }
                Status::Error(e) => {
                    return Err(format!("Installation failed: {}", e).into());
                }
            }
        }

        Ok(())
    }).await
}
```

### 複数の非同期タスク

```rust
use tokio::try_join;

pub async fn run(args: MyArgs) -> CommandResult {
    let udid = args.device.udid.as_deref();

    // 並列実行
    let (screenshot, accessibility) = try_join!(
        take_screenshot(udid),
        get_accessibility_info(udid)
    )?;

    println!("Screenshot: {} bytes", screenshot.len());
    println!("Accessibility: {} elements", accessibility.len());

    Ok(())
}

async fn take_screenshot(udid: Option<&str>) -> CommandResult<Vec<u8>> {
    with_xcuitest(|mut client| async move {
        client.screenshot().await
    }).await
}

async fn get_accessibility_info(udid: Option<&str>) -> CommandResult<String> {
    with_xcuitest(|mut client| async move {
        client.accessibility_info(None, true).await
    }).await
}
```

## プラットフォーム分岐パターン

### 基本形

```rust
use agent_mobile_core::Platform;
use agent_mobile_gateway::DeviceResolver;

pub async fn run(args: MyArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    match platform {
        Platform::Ios => run_ios(args.device.udid.as_deref()).await,
        Platform::Android => run_android(args.device.udid.as_deref()).await,
    }
}

async fn run_ios(udid: Option<&str>) -> CommandResult {
    with_xcuitest(|mut client| async move {
        // iOS実装
        Ok(())
    }).await
}

async fn run_android(udid: Option<&str>) -> CommandResult {
    // Android実装
    Ok(())
}
```

### プラットフォーム固有のエラー

```rust
pub async fn run(args: MyArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    match platform {
        Platform::Ios => run_ios(args).await,
        Platform::Android => {
            Err("Android not supported yet for this command".into())
        }
    }
}
```

### プラットフォーム共通処理

```rust
pub async fn run(args: TapArgs) -> CommandResult {
    let platform = match args.device.udid.as_deref() {
        Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
        None => DeviceResolver::detect_platform().await?,
    };

    // 1. 共通処理: 座標解決
    let (x, y) = resolve_coords(&args.target, platform, args.device.udid.as_deref()).await?;

    // 2. プラットフォーム別処理: タップ実行
    execute_tap(platform, args.device.udid.as_deref(), x, y).await
}

async fn execute_tap(platform: Platform, udid: Option<&str>, x: f64, y: f64) -> CommandResult {
    match platform {
        Platform::Ios => {
            with_xcuitest(|mut client| async move {
                let events = tap_events(x, y);
                client.hid(events).await?;
                Ok(())
            }).await
        }
        Platform::Android => {
            use agent_mobile_platform_android::adb::input;
            input::tap(udid, x, y).await?;
            Ok(())
        }
    }
}
```

## ストリーミングパターン

### コンソールログストリーミング (iOS: simctl spawn log / Android: LogcatStream)

```rust
use agent_mobile_gateway::stream_console_logs;
use agent_mobile_core::OutputWriter;
use tokio::sync::watch;
use crate::helpers::signal::setup_ctrl_c_handler;

pub async fn run(args: ConsoleArgs) -> CommandResult {
    let (stop_tx, stop_rx) = watch::channel(false);
    setup_ctrl_c_handler(stop_tx.clone());

    let writer = match &args.output {
        Some(path) => OutputWriter::tee_from_path(path)?,
        None => OutputWriter::stdout(),
    };

    // Gateway層でiOS/Android共通のログストリーミング
    stream_console_logs(platform, udid, writer, stop_rx).await?;
    Ok(())
}
```

### ADB native protocol 操作

```rust
use agent_mobile_platform_android::adb;

// スクリーンショット取得（PNG直接取得、JPEG変換対応）
let png_data = adb::screenshot::screenshot(serial).await?;
let jpeg_data = adb::screenshot::screenshot_bytes(serial, ImageFormat::Jpeg).await?;

// UI階層取得（XML解析）
let xml = adb::uiautomator::dump_ui(serial).await?;
let elements = adb::uiautomator::parse_ui_hierarchy(&xml)?;

// ログストリーミング
let mut logcat = adb::logcat::LogcatStream::new(serial).await?;
```

### XCUITest Runner HTTP パターン

```rust
// Screenshot (バイナリレスポンス)
let png_bytes = client.screenshot().await?;

// アプリ操作 (JSON API)
client.launch_app(bundle_id).await?;
client.terminate_app(bundle_id).await?;

// クリップボード操作
client.set_clipboard(text).await?;
let content = client.get_clipboard().await?;
```

## まとめ

**最頻出パターン**:
1. `with_xcuitest()` + `DeviceArgs` flatten
2. `CommandResult` + `?` 演算子
3. プラットフォーム分岐（`match Platform`）
4. アクション可能なエラーメッセージ
5. JSON出力対応（`DeviceFormatArgs`）

**ベストプラクティス**:
- エラーメッセージは明確かつアクション可能に
- 引数検証は早期に行う
- iOS操作は `with_xcuitest()` パターンを使用（UDIDパラメータなし）
- ログストリーミングは Gateway層 `stream_console_logs()` を使用
- Ctrl+Cハンドリングはtokio::select!で実装
- JSON出力時はエラーも構造化する

**関連ファイル**:
- `src/helpers/client.rs`: with_xcuitest実装
- `src/helpers/common_args.rs`: DeviceArgs定義
- `src/core/`: 実装例
