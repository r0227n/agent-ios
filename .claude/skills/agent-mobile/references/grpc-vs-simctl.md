# gRPC vs simctl: Architecture Deep Dive

agent-mobile の iOS 実装におけるハイブリッドアーキテクチャの技術的解説。

## Overview

agent-mobile の iOS 実装は、**idb gRPC** と **xcrun simctl** の2つの技術を戦略的に組み合わせたハイブリッドアプローチを採用しています。この設計により、各技術の強みを活かしながら、弱点を補完することができます。

## Design Philosophy

### 原則: "Right Tool for the Right Job"

機能ごとに最適な実装方法を選択します:

```
┌─────────────────────────────────────────┐
│ 判断マトリックス                          │
├─────────────────────────────────────────┤
│ proto/idb.proto に RPC 定義あり?         │
│   YES → idb gRPC                        │
│   NO  → xcrun simctl                    │
│                                         │
│ 実機対応が必要?                          │
│   YES → idb gRPC                        │
│   NO  → どちらでも可                     │
│                                         │
│ ストリーミングが必要?                     │
│   YES → idb gRPC                        │
│   NO  → どちらでも可                     │
│                                         │
│ ライフサイクル管理?                       │
│   YES → xcrun simctl                    │
│   NO  → idb gRPC                        │
└─────────────────────────────────────────┘
```

## idb gRPC

### What is idb?

**idb (iOS Development Bridge)** は Facebook/Meta が開発した iOS デバイス/シミュレータ管理ツールです。Python 実装の CLI と、Objective-C/Swift で書かれた companion デーモンで構成されます。

### Architecture

```
agent-mobile (Rust)
    ↓
idb gRPC Client (tonic)
    ↓ [gRPC over HTTP/2]
idb_companion (ObjC/Swift) - Port 10882
    ↓
iOS Frameworks:
  - CoreSimulator
  - XCTest
  - Accessibility
  - Instruments
```

### Advantages

1. **型安全**: Protocol Buffers による厳密な型定義
2. **ストリーミング**: 長時間の操作（ログ、インストール）で進捗を報告
3. **高度な機能**: XCTest、デバッガアタッチ、Instruments 統合
4. **実機対応**: USB/ネットワーク経由でペアリングされた実機にアクセス
5. **非同期**: gRPC の非同期 API で並行実行が容易

### Limitations

1. **companion プロセス必須**: idb_companion がクラッシュするとすべて停止
2. **proto 制約**: proto に定義されていない機能は使えない
3. **オーバーヘッド**: gRPC 通信のレイテンシ（約10-20ms）
4. **バージョン互換性**: idb のバージョンアップで proto が変わる可能性

### Usage in agent-mobile

```rust
// src/platform/ios/grpc/client.rs

use crate::grpc::CompanionClient;

pub async fn accessibility_info(
    client: &mut CompanionClient,
) -> Result<String> {
    let request = AccessibilityInfoRequest {
        format: Some(Format::Json as i32),
        nested: Some(true),
        point: None,
    };

    let response = client.accessibility_info(request).await?;
    Ok(response.into_inner().json)
}
```

**使用箇所:**
- `snapshot`: UI 階層取得（accessibility_info RPC）
- `tap/fill/type`: HID イベント送信（hid RPC）
- `screenshot`: フレームバッファ取得（screenshot RPC）
- `app launch/terminate`: アプリ管理（launch/terminate RPC）
- `console`: ログストリーミング（log RPC）
- `record`: 画面録画（record RPC）

## xcrun simctl

### What is simctl?

**simctl** は Apple が提供する公式のシミュレータ管理コマンドラインツールです。Xcode に同梱されており、追加インストール不要です。

### Architecture

```
agent-mobile (Rust)
    ↓
std::process::Command
    ↓ [子プロセス起動]
xcrun simctl (Xcode CLI)
    ↓
CoreSimulator.framework
    ↓
iOS Simulator
```

### Advantages

1. **公式ツール**: Apple サポート、安定性が高い
2. **追加依存なし**: Xcode があれば動作
3. **シンプル**: 単純なコマンド実行
4. **完全な機能**: すべてのシミュレータ操作をサポート
5. **ドキュメント**: `man simctl` で詳細な説明

### Limitations

1. **シミュレータ限定**: 実機では使用不可
2. **ストリーミング不可**: 進捗報告なし
3. **テキスト出力**: JSON パースが必要
4. **同期実行**: 非同期処理は手動で実装
5. **エラーハンドリング**: exit code のみ

### Usage in agent-mobile

```rust
// src/platform/ios/simctl/management.rs

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
```

**使用箇所:**
- `device boot`: シミュレータ起動（simctl boot）
- `device shutdown`: シミュレータ停止（simctl shutdown）
- `device pbcopy/pbpaste`: クリップボード操作（simctl pbcopy/pbpaste）
- Permission fallback: approve エラー時（simctl privacy）

## Feature Mapping

### 完全な機能マトリックス

| 機能 | idb gRPC | xcrun simctl | 現在の選択 | 理由 |
|------|----------|-------------|-----------|------|
| **Device Lifecycle** |
| List Devices | ✅ list_targets | ✅ list | gRPC | JSON 出力が構造化 |
| Boot Simulator | ❌ | ✅ boot | **simctl** | proto に boot RPC なし |
| Shutdown Simulator | ❌ | ✅ shutdown | **simctl** | proto に shutdown RPC なし |
| Create Simulator | ❌ | ✅ create | **simctl** | gRPC 未対応 |
| Delete Simulator | ❌ | ✅ delete | **simctl** | gRPC 未対応 |
| Clone Simulator | ❌ | ✅ clone | **simctl** | gRPC 未対応 |
| Erase Simulator | ❌ | ✅ erase | **simctl** | gRPC 未対応 |
| **UI Interaction** |
| Accessibility Info | ✅ accessibility_info | ❌ | **gRPC** | simctl では不可 |
| HID Events (tap/type) | ✅ hid | ❌ | **gRPC** | simctl では不可 |
| Screenshot | ✅ screenshot | ✅ io screenshot | **gRPC** | Framebuffer 直接アクセス |
| Record Video | ✅ record | ✅ io recordVideo | **gRPC** | ストリーミング対応 |
| **Application Management** |
| Launch App | ✅ launch | ✅ launch | **gRPC** | ストリーミング、進捗対応 |
| Terminate App | ✅ terminate | ✅ terminate | **gRPC** | 統一 API |
| Install App | ✅ install | ✅ install | **gRPC** | 進捗ストリーミング |
| Uninstall App | ✅ uninstall | ✅ uninstall | **gRPC** | 統一 API |
| List Apps | ✅ list_apps | ✅ listapps | **gRPC** | JSON 出力が構造化 |
| **Permissions** |
| Grant Permission | ✅ approve | ✅ privacy grant | **gRPC + fallback** | エラー時 simctl |
| Revoke Permission | ✅ revoke | ✅ privacy revoke | **gRPC + fallback** | エラー時 simctl |
| Reset Permission | ❌ | ✅ privacy reset | **simctl** | gRPC 未対応 |
| **System** |
| Clipboard Copy | ❌ | ✅ pbcopy | **simctl** | gRPC 未対応 |
| Clipboard Paste | ❌ | ✅ pbpaste | **simctl** | gRPC 未対応 |
| Notification | ✅ send_notification | ✅ push | **gRPC** | 統一 API |
| Status Bar | ❌ | ✅ status_bar | **simctl** | gRPC 未対応 |
| **Advanced** |
| XCTest | ✅ xctest_* | ❌ | **gRPC** | simctl では不可 |
| Debugserver | ✅ debugserver | ❌ | **gRPC** | simctl では不可 |
| Instruments | ✅ instruments | ❌ | **gRPC** | simctl では不可 |
| File Operations | ✅ file_* | ✅ get_app_container | **gRPC** | 統一 API |
| Log Stream | ✅ log | ✅ spawn log | **gRPC** | ストリーミング対応 |

**凡例:**
- ✅ = サポートあり
- ❌ = サポートなし

## Hybrid Patterns

### Pattern 1: gRPC First, simctl Fallback

権限管理で使用されるパターン:

```rust
// src/app/mod.rs

async fn execute_grant(
    platform: Platform,
    udid: &str,
    bundle_id: &str,
    permission: &str,
) -> CommandResult {
    // 1. Try idb gRPC first
    let result = with_client(Some(udid), |mut client| async move {
        let perm_id = ios_permission_to_id(permission);
        client.approve(bundle_id, vec![perm_id], None).await
    })
    .await;

    match result {
        Ok(_) => {
            println!("Granted {} to {}", permission, bundle_id);
            Ok(())
        }
        Err(_) => {
            // 2. Fall back to simctl on error
            use crate::platform::ios::simctl::management;
            management::privacy_grant(udid, permission, bundle_id)?;
            println!("Granted {} to {} (via simctl)", permission, bundle_id);
            Ok(())
        }
    }
}
```

**利点:**
- 通常は高機能な gRPC を使用
- エラー時は安定した simctl にフォールバック
- ユーザーは内部実装を意識不要

### Pattern 2: Exclusive Use

機能が一方にしかない場合:

```rust
// Boot: simctl のみ
pub fn boot(udid: &str) -> Result<()> {
    Command::new("xcrun")
        .args(["simctl", "boot", udid])
        .output()?;
    Ok(())
}

// Accessibility Info: gRPC のみ
pub async fn snapshot(client: &mut CompanionClient) -> Result<String> {
    client.accessibility_info(None, true).await
}
```

### Pattern 3: Best-of-Breed

両方で可能だが、一方が明らかに優れている場合:

```rust
// Screenshot: gRPC を選択（Framebuffer 直接アクセス）
pub async fn screenshot(client: &mut CompanionClient) -> Result<Vec<u8>> {
    let response = client.screenshot(ScreenshotRequest {
        format: Some(Format::Png as i32),
    }).await?;

    Ok(response.into_inner().image_data)
}

// simctl でも可能だが使わない:
// xcrun simctl io <udid> screenshot output.png
```

## Implementation Details

### gRPC Connection Management

```rust
// src/grpc/mod.rs

pub struct CompanionClient {
    inner: IdbClient<tonic::transport::Channel>,
}

impl CompanionClient {
    pub async fn connect(address: &str) -> Result<Self> {
        let endpoint = Endpoint::from_shared(format!("http://{}", address))?
            .connect_timeout(Duration::from_secs(5));

        let channel = endpoint.connect().await?;
        let inner = IdbClient::new(channel);

        Ok(Self { inner })
    }
}
```

**Connection Pooling:**
agent-mobile は `with_client()` ヘルパーで接続を管理:

```rust
// src/helpers.rs

pub async fn with_client<F, Fut, T>(
    udid: Option<&str>,
    f: F,
) -> Result<T>
where
    F: FnOnce(CompanionClient) -> Fut,
    Fut: Future<Output = Result<T>>,
{
    let address = resolve_companion_address(udid)?;
    let client = CompanionClient::connect(&address).await?;
    f(client).await
}
```

### simctl Error Handling

```rust
// src/platform/ios/simctl/management.rs

fn parse_simctl_error(stderr: &str) -> String {
    if stderr.contains("Unable to boot device in current state: Booted") {
        "Device is already booted".to_string()
    } else if stderr.contains("No such file or directory") {
        "Device not found".to_string()
    } else {
        stderr.to_string()
    }
}
```

## Performance Comparison

### Snapshot (UI Hierarchy)

**idb gRPC:**
```rust
// ~500ms (ネットワーク + JSON シリアライズ)
let json = client.accessibility_info(None, true).await?;
```

**simctl (不可):**
simctl には accessibility_info 相当の機能なし。

### Screenshot

**idb gRPC:**
```rust
// ~200ms (Framebuffer 直接)
let image = client.screenshot(ScreenshotRequest::default()).await?;
```

**simctl:**
```bash
# ~300ms (ファイル I/O)
xcrun simctl io <udid> screenshot /tmp/screenshot.png
```

**→ gRPC が 1.5x 速い**

### App Launch

**idb gRPC:**
```rust
// ~2s (ストリーミングで進捗報告)
let mut stream = client.launch(config, false, stop_rx).await?;
while let Some(msg) = stream.message().await? {
    println!("Launch progress: {:?}", msg);
}
```

**simctl:**
```bash
# ~2s (進捗報告なし)
xcrun simctl launch <udid> <bundle-id>
```

**→ 速度は同等だが、gRPC は進捗表示可能**

## Decision Tree

新しい機能を追加する際の判断フロー:

```
┌─ 新機能を追加したい
│
├─ proto/idb.proto に RPC 定義がある?
│  ├─ YES → idb gRPC で実装
│  └─ NO ─┐
│         │
│         ├─ ストリーミングが必要?
│         │  ├─ YES → proto に RPC 追加を検討
│         │  └─ NO ─┐
│         │         │
│         │         ├─ 実機対応が必要?
│         │         │  ├─ YES → idb Python CLI を経由
│         │         │  └─ NO ─┐
│         │         │         │
│         │         │         ├─ シミュレータライフサイクル?
│         │         │         │  ├─ YES → xcrun simctl
│         │         │         │  └─ NO → どちらでも可
│         │         │         │         (simctl を推奨: シンプル)
│         │         │         │
│         │         │         └─ 実装完了
```

## Real-World Examples

### Example 1: Clipboard Support

**要件:**
- クリップボードにテキストをコピー/ペースト
- シミュレータのみ対応

**判断:**
```
proto に API ある? → NO
ストリーミング必要? → NO
実機対応必要? → NO
ライフサイクル? → NO (が、simctl にしかない)
→ xcrun simctl を使用
```

**実装:**
```rust
// src/platform/ios/simctl/management.rs

pub fn pbcopy(udid: &str, text: &str) -> Result<()> {
    let mut child = Command::new("xcrun")
        .args(["simctl", "pbcopy", udid])
        .stdin(Stdio::piped())
        .spawn()?;

    child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
    child.wait()?;
    Ok(())
}

pub fn pbpaste(udid: &str) -> Result<String> {
    let output = Command::new("xcrun")
        .args(["simctl", "pbpaste", udid])
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

### Example 2: Permission Management

**要件:**
- アプリに権限を付与/取り消し
- 両方のAPIで可能だが、gRPC でエラーが発生することがある

**判断:**
```
proto に API ある? → YES (approve/revoke)
→ まず gRPC を試す
→ エラー時は simctl にフォールバック
```

**実装:**
```rust
// src/app/mod.rs (Pattern 1 参照)
```

### Example 3: UI Snapshot

**要件:**
- UI 階層を取得して要素参照を生成
- ストリーミングは不要

**判断:**
```
proto に API ある? → YES (accessibility_info)
→ idb gRPC を使用
(simctl には同等機能なし)
```

**実装:**
```rust
// src/snapshot/mod.rs

let json_str = client.accessibility_info(None, true).await?;
let json: serde_json::Value = serde_json::from_str(&json_str)?;
let elements = extractor::extract_ios_elements(&json);
```

## Maintenance Considerations

### When to Update

**idb proto 更新時:**
```bash
# 1. idb サブモジュールを更新
cd idb
git pull origin main

# 2. proto を再生成
./scripts/regenerate_proto.sh

# 3. Rust コードを更新（型が変わった場合）
cargo build
```

**simctl API 変更時:**
```bash
# Xcode リリースノートを確認
# 通常は後方互換性が保たれる
```

### Testing Strategy

```rust
// tests/integration/ios_hybrid_test.rs

#[tokio::test]
async fn test_hybrid_permission_grant() {
    // gRPC で権限付与
    let result_grpc = grant_via_grpc("camera", "com.test.app").await;

    // simctl で確認
    let status = check_via_simctl("camera", "com.test.app").await;
    assert_eq!(status, "authorized");
}
```

## Related Concepts

- [Platform Differences](platform-differences.md) - iOS vs Android の比較
- [Troubleshooting](troubleshooting.md) - 両方のアプローチのエラー解決
- [CLAUDE.md - cli-feature.md](../../.claude/rules/cli-feature.md) - 実装ガイドライン

---

**Last Updated**: 2026-01-23
**Related Files:**
- `/Users/r0227n/Dev/agent-mobile/.claude/rules/cli-feature.md` - 実装の判断基準
- `/Users/r0227n/Dev/agent-mobile/proto/idb.proto` - gRPC プロトコル定義
