# iOS プラットフォーム実装判断基準

agent-mobile の iOS実装における **idb gRPC** と **xcrun simctl** の選択基準を詳細に解説します。

> **注**: このファイルは `.claude/rules/cli-feature.md` の内容を統合・拡充したものです。

## 目次

- [結論](#結論)
- [判断基準](#判断基準)
- [判断フローチャート](#判断フローチャート)
- [機能別の推奨](#機能別の推奨)
- [各アプローチの特徴](#各アプローチの特徴)
- [idb.proto RPC一覧](#idbproto-rpc一覧)
- [xcrun simctl コマンド一覧](#xcrun-simctl-コマンド一覧)
- [重複機能がある場合の判断](#重複機能がある場合の判断)
- [機能別の実装状況](#機能別の実装状況)
- [ハイブリッド実装パターン](#ハイブリッド実装パターン)
- [判断履歴](#判断履歴)
- [実装例](#実装例)

## 結論

**ハイブリッドアプローチを推奨**: 機能に応じて xcrun simctl と idb gRPC を使い分ける。

## 判断基準

| 条件 | 選択 |
|------|------|
| `idb.proto` に RPC 定義あり | → **idb gRPC** |
| シミュレータライフサイクル管理 | → **xcrun simctl** |
| 実機対応が必要 | → **idb gRPC** |
| ストリーミングが必要 | → **idb gRPC** |
| proto に API がない | → **xcrun simctl** |

## 判断フローチャート

```text
新機能 <feature> を追加

1. proto/idb.proto を確認
   │
   ├─ RPC定義あり？
   │  │
   │  ├─ Yes → idb gRPC 実装
   │  │  │
   │  │  ├─ Unary RPC？ → with_client() 使用
   │  │  └─ Streaming RPC？ → with_client_streaming() 使用
   │  │
   │  └─ No → xcrun simctl 実装
   │     │
   │     └─ simctl::management モジュール拡張
   │
2. 実機対応が必要？
   │
   ├─ Yes → idb gRPC 実装（simctlは実機非対応）
   └─ No → simctl も選択肢
   │
3. ストリーミングが必要？
   │
   ├─ Yes → idb gRPC 実装
   └─ No → simctl も選択肢
   │
4. 両方で実装可能？
   │
   └─ ハイブリッド実装
      ├─ gRPC 優先
      └─ エラー時に simctl fallback
```

## 機能別の推奨

### idb gRPC を使う

| 機能 | proto RPC | 理由 |
|------|----------|------|
| HID 入力 (tap/swipe/text) | `hid` | 精密な制御、ストリーミング |
| アクセシビリティ | `accessibility_info` | simctlでは不可 |
| スクリーンショット | `screenshot` (内部実装) | Framebuffer 直接アクセス |
| ファイル操作 | `ls`, `mkdir`, `mv`, `rm`, `pull`, `push` | 統一インターフェース |
| アプリ操作 | `install`, `launch`, `terminate`, `uninstall` | 進捗ストリーミング |
| ログ取得 | `log` | リアルタイムストリーミング |
| 動画録画 | `record` | ストリーミング |
| デバッグサーバー | `debugserver` | ストリーミング |
| XCTest | `xctest_run`, `xctest_list_tests` | 進捗ストリーミング |
| 位置情報 | `set_location` | 統一API |
| 通知送信 | `send_notification` | 統一API |
| 設定管理 | `setting`, `get_setting` | 統一API |
| 権限管理 | `approve`, `revoke` | 統一API |

### xcrun simctl を使う

| 機能 | コマンド | 理由 |
|------|---------|------|
| シミュレータ起動 | `xcrun simctl boot <udid>` | proto に boot RPC なし |
| シミュレータ停止 | `xcrun simctl shutdown <udid>` | proto に shutdown RPC なし |
| シミュレータ作成 | `xcrun simctl create <name> <type>` | ライフサイクル管理 |
| シミュレータ削除 | `xcrun simctl delete <udid>` | ライフサイクル管理 |
| シミュレータ消去 | `xcrun simctl erase <udid>` | ライフサイクル管理 |
| **クリップボード** | `xcrun simctl pbcopy/pbpaste <udid>` | **proto に API なし** |
| プライバシー設定 | `xcrun simctl privacy <udid> grant/revoke` | gRPCでエラー時のfallback |

## 各アプローチの特徴

### idb gRPC

**利点:**
- **型安全**: Protocol Buffers による型定義
- **ストリーミング対応**: 長時間実行、リアルタイムデータ
- **実機対応**: シミュレータ/実機両方に対応
- **高度な機能**: アクセシビリティ、HID入力など
- **進捗表示**: アプリインストール、XCTest実行時の進捗
- **統一API**: ファイル、アプリ、設定などの統一インターフェース

**欠点:**
- **idb_companion 必須**: デーモンプロセスが必要
- **接続オーバーヘッド**: gRPC接続の初期化コスト
- **一部機能なし**: boot、shutdown、clipboard など
- **実装複雑度**: ストリーミング処理、エラーハンドリング

**使用場面:**
- UI自動化、入力操作
- アプリのインストール、起動、終了
- ファイル転送、ログ取得
- 実機操作が必要な場合

### xcrun simctl

**利点:**
- **Apple 公式**: 安定、信頼性高い
- **追加依存なし**: Xcodeに付属
- **シンプル**: コマンド実行だけ
- **シミュレータ管理**: boot、shutdown、create、delete

**欠点:**
- **シミュレータ限定**: 実機では使用不可
- **ストリーミング不可**: 出力は一括取得のみ
- **機能限定**: HID、アクセシビリティなどは不可
- **パース必要**: 出力は文字列で返される

**使用場面:**
- シミュレータのライフサイクル管理
- protoに定義がない機能（clipboard）
- idb_companionが不可の状況

## idb.proto RPC一覧

`proto/idb.proto` で定義されているRPCをカテゴリ別にリスト化します。

### Management（管理）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `describe` | `TargetDescriptionRequest` | `TargetDescriptionResponse` | デバイス情報取得 |
| `debugserver` | `stream DebugServerRequest` | `stream DebugServerResponse` | デバッグサーバー起動 |
| `dap` | `stream DapRequest` | `stream DapResponse` | Debug Adapter Protocol |

### Interaction（操作）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `accessibility_info` | `AccessibilityInfoRequest` | `AccessibilityInfoResponse` | アクセシビリティ情報取得 |
| `focus` | `FocusRequest` | `FocusResponse` | アプリをフォーカス |
| `hid` | `stream HIDEvent` | `HIDResponse` | HID入力（tap、swipe、text） |
| `open_url` | `OpenUrlRequest` | `OpenUrlResponse` | URL開く |
| `set_location` | `SetLocationRequest` | `SetLocationResponse` | 位置情報設定 |
| `send_notification` | `SendNotificationRequest` | `SendNotificationResponse` | 通知送信 |
| `simulate_memory_warning` | `SimulateMemoryWarningRequest` | `SimulateMemoryWarningResponse` | メモリ警告シミュレート |

### Settings（設定）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `approve` | `ApproveRequest` | `ApproveResponse` | 権限承認 |
| `revoke` | `RevokeRequest` | `RevokeResponse` | 権限取り消し |
| `clear_keychain` | `ClearKeychainRequest` | `ClearKeychainResponse` | キーチェーンクリア |
| `contacts_update` | `ContactsUpdateRequest` | `ContactsUpdateResponse` | 連絡先更新 |
| `contacts_clear` | `ContactsClearRequest` | `ContactsClearResponse` | 連絡先クリア |
| `photos_clear` | `PhotosClearRequest` | `PhotosClearResponse` | 写真クリア |
| `setting` | `SettingRequest` | `SettingResponse` | 設定変更 |
| `get_setting` | `GetSettingRequest` | `GetSettingResponse` | 設定取得 |
| `list_settings` | `ListSettingRequest` | `ListSettingResponse` | 設定一覧 |

### App（アプリ）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `install` | `stream InstallRequest` | `stream InstallResponse` | アプリインストール（進捗） |
| `launch` | `stream LaunchRequest` | `stream LaunchResponse` | アプリ起動（wait_for対応） |
| `list_apps` | `ListAppsRequest` | `ListAppsResponse` | アプリ一覧 |
| `terminate` | `TerminateRequest` | `TerminateResponse` | アプリ終了 |
| `uninstall` | `UninstallRequest` | `UninstallResponse` | アプリアンインストール |

### Media（メディア）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `add_media` | `stream AddMediaRequest` | `AddMediaResponse` | メディア追加 |
| `record` | `stream RecordRequest` | `stream RecordResponse` | 画面録画 |
| `video_stream` | `stream VideoStreamRequest` | `stream VideoStreamResponse` | 動画ストリーミング |

### File Operations（ファイル操作）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `ls` | `LsRequest` | `LsResponse` | ディレクトリ一覧 |
| `mkdir` | `MkdirRequest` | `MkdirResponse` | ディレクトリ作成 |
| `mv` | `MvRequest` | `MvResponse` | ファイル移動 |
| `rm` | `RmRequest` | `RmResponse` | ファイル削除 |
| `pull` | `PullRequest` | `stream PullResponse` | ファイル取得 |
| `push` | `stream PushRequest` | `PushResponse` | ファイル送信 |
| `tail` | `stream TailRequest` | `stream TailResponse` | ファイル監視 |

### Crash Operations（クラッシュ操作）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `crash_delete` | `CrashLogQuery` | `CrashLogResponse` | クラッシュログ削除 |
| `crash_list` | `CrashLogQuery` | `CrashLogResponse` | クラッシュログ一覧 |
| `crash_show` | `CrashShowRequest` | `CrashShowResponse` | クラッシュログ表示 |

### XCTest Operations（XCTest操作）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `xctest_list_bundles` | `XctestListBundlesRequest` | `XctestListBundlesResponse` | テストバンドル一覧 |
| `xctest_list_tests` | `XctestListTestsRequest` | `XctestListTestsResponse` | テスト一覧 |
| `xctest_run` | `XctestRunRequest` | `stream XctestRunResponse` | テスト実行 |

### Logging（ログ）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `log` | `LogRequest` | `stream LogResponse` | ログストリーミング |

### Instruments（計測）

| RPC | Request | Response | 用途 |
|-----|---------|----------|------|
| `instruments_run` | `stream InstrumentsRunRequest` | `stream InstrumentsRunResponse` | Instruments実行 |
| `xctrace_record` | `stream XctraceRecordRequest` | `stream XctraceRecordResponse` | xctrace録画 |

## xcrun simctl コマンド一覧

`xcrun simctl` で使用可能なコマンド（agent-mobileで関連するもの）:

### ライフサイクル管理

| コマンド | 引数 | 説明 |
|---------|------|------|
| `boot` | `<udid>` | シミュレータ起動 |
| `shutdown` | `<udid>` | シミュレータ停止 |
| `create` | `<name> <device-type> <runtime>` | シミュレータ作成 |
| `delete` | `<udid>` | シミュレータ削除 |
| `erase` | `<udid>` | シミュレータ消去 |
| `clone` | `<udid> <new-name>` | シミュレータクローン |
| `list` | `[devices|runtimes|devicetypes]` | デバイス一覧 |

### クリップボード

| コマンド | 引数 | 説明 |
|---------|------|------|
| `pbcopy` | `<udid>` | クリップボードにコピー（stdin） |
| `pbpaste` | `<udid>` | クリップボードからペースト（stdout） |

### プライバシー設定

| コマンド | 引数 | 説明 |
|---------|------|------|
| `privacy` | `<udid> grant <service> <bundle>` | 権限付与 |
| `privacy` | `<udid> revoke <service> <bundle>` | 権限取り消し |
| `privacy` | `<udid> reset <service> [bundle]` | 権限リセット |

**サービス例**: `photos`, `camera`, `contacts`, `location`, `microphone`, `motion`

### その他

| コマンド | 引数 | 説明 |
|---------|------|------|
| `spawn` | `<udid> <command>` | シミュレータ内でコマンド実行 |
| `openurl` | `<udid> <url>` | URL開く |
| `addmedia` | `<udid> <path>` | メディア追加 |
| `get_app_container` | `<udid> <bundle> [data|app|groups]` | アプリコンテナパス取得 |

## 重複機能がある場合の判断

両方で実装可能な機能の場合、以下の優先順位で選択:

### 1. idb gRPC を優先するケース

| 理由 | 例 |
|------|-----|
| ストリーミングが必要 | アプリインストール（進捗表示）|
| 型安全が重要 | アプリ起動（エラーハンドリング）|
| 実機対応が必要 | スクリーンショット |
| 高度な機能が必要 | アクセシビリティ情報 |
| 統一API | ファイル操作、設定管理 |

### 2. simctl をフォールバックとして使うケース

| 状況 | 例 |
|------|-----|
| idb gRPC でエラー | 権限管理（SQLite スキーマエラー）|
| idb_companion 不可時 | 起動前の操作 |
| protoに実装がない | クリップボード操作 |

### 3. simctl のみを使うケース

| 理由 | 例 |
|------|-----|
| proto に API がない | シミュレータ作成/クローン |
| ライフサイクル管理 | boot/shutdown（現状） |

## 機能別の実装状況

agent-mobile における現在の実装状況と選択理由:

| 機能 | idb gRPC | xcrun simctl | 現在の選択 | 理由 |
|------|---------|-------------|-----------|------|
| **アクセシビリティ** | ✓ | ✗ | **gRPC** | simctl では不可 |
| **アプリ起動/終了** | ✓ | ✓ | **gRPC** | ストリーミング対応、wait_for |
| **アプリインストール** | ✓ | ✓ | **gRPC** | 進捗表示 |
| **スクリーンショット** | ✓ | ✓ | **gRPC** | Framebuffer 直接 |
| **権限管理** | ✓ | ✓ | **gRPC** (+ simctl fallback) | エラー時切替 |
| **通知送信** | ✓ | ✓ | **gRPC** | 統一 API |
| **シミュレータ起動** | △ | ✓ | **simctl** | proto に boot RPC なし |
| **シミュレータ作成** | ✗ | ✓ | **simctl** | gRPC 未対応 |
| **クリップボード** | ✗ | ✓ | **simctl** | gRPC 未対応 |
| **HID 入力** | ✓ | ✗ | **gRPC** | simctl では不可 |
| **ファイル操作** | ✓ | △ | **gRPC** | 統一API、ストリーミング |
| **ログ取得** | ✓ | △ | **gRPC** | リアルタイムストリーミング |
| **動画録画** | ✓ | △ | **gRPC** | ストリーミング |
| **位置情報設定** | ✓ | ✓ | **gRPC** | 統一API |

## ハイブリッド実装パターン

### パターン1: gRPC優先、fallback で simctl

権限管理の例（gRPCでSQLiteエラーが発生する場合）:

```rust
pub async fn grant_permission(
    udid: &str,
    bundle: &str,
    perm: Permission
) -> Result<()> {
    // 1. まず idb gRPC を試す
    match idb_client.approve(bundle, perm).await {
        Ok(_) => Ok(()),
        Err(e) if e.is_schema_error() => {
            // 2. 失敗時は simctl にフォールバック
            eprintln!("gRPC failed, falling back to simctl: {}", e);
            simctl::privacy_grant(udid, perm, bundle)
        }
        Err(e) => Err(e),
    }
}
```

### パターン2: 環境に応じた選択

シミュレータ起動時のみsimctl、それ以外はgRPC:

```rust
pub async fn run(args: MyArgs) -> CommandResult {
    // シミュレータ起動はsimctlのみ
    if args.command == "boot" {
        return simctl::boot(&args.udid);
    }

    // その他の操作はgRPC
    with_client(Some(&args.udid), |mut client| async move {
        // gRPC操作
        Ok(())
    }).await
}
```

### パターン3: 機能検出

protoに定義があるか動的に判断:

```rust
pub async fn execute_feature(feature: &str, udid: &str) -> CommandResult {
    match feature {
        "accessibility" | "hid" | "log" => {
            // gRPCのみ対応
            with_client(Some(udid), |mut client| async move {
                // gRPC実装
                Ok(())
            }).await
        }
        "boot" | "shutdown" | "create" | "clipboard" => {
            // simctlのみ対応
            simctl::execute(feature, udid)
        }
        "permission" => {
            // ハイブリッド（gRPC優先、fallback）
            grant_permission(udid, bundle, perm).await
        }
        _ => Err("Unknown feature".into()),
    }
}
```

## 判断履歴

既存機能がなぜ特定の実装を選んだかの履歴:

### tap/swipe/scroll (HID入力) → idb gRPC

**理由**:
- 精密な座標制御が必要
- タッチイベントのストリーミング
- simctlにはHID APIがない

**実装場所**: `src/core/tap.rs`, `src/idb/hid/`

### accessibility_info → idb gRPC

**理由**:
- simctlにはアクセシビリティAPIがない
- UI要素の階層情報が必要
- JSON形式で構造化データ取得

**実装場所**: `src/idb/accessibility.rs`

### boot/shutdown → xcrun simctl

**理由**:
- proto/idb.protoにboot/shutdown RPCがない
- シミュレータライフサイクル管理は公式ツールが適切
- idb_companionは起動後にしか接続できない

**実装場所**: `src/idb/target/boot_shutdown.rs`

### screenshot → idb gRPC

**理由**:
- Framebufferへの直接アクセス
- 実機対応が必要
- 高品質な画像取得

**実装場所**: `src/core/screenshot.rs`

### file operations (ls/mkdir/mv/rm/pull/push) → idb gRPC

**理由**:
- 統一されたファイル操作API
- ストリーミング対応（push/pull）
- アプリコンテナへのアクセス

**実装場所**: `src/idb/file/`

### clipboard → xcrun simctl

**理由**:
- proto/idb.protoにclipboard APIがない
- simctlのpbcopy/pbpasteで実現可能

**実装予定**: `src/core/clipboard.rs` (未実装)

### permission → idb gRPC (fallback simctl)

**理由**:
- gRPCのapprove/revoke RPCを優先
- SQLiteスキーマエラー時にsimctl fallback
- 両方の利点を活かす

**実装場所**: `src/idb/permissions.rs`

## 実装例

### idb gRPC実装例

```rust
use crate::cli::helpers::{with_client, CommandResult};

pub async fn run(udid: Option<String>) -> CommandResult {
    with_client(udid.as_deref(), |mut client| async move {
        // RPC呼び出し
        let response = client.accessibility_info(None, true).await?;
        println!("{}", response);
        Ok(())
    }).await
}
```

### xcrun simctl実装例

```rust
use std::process::Command;
use crate::cli::helpers::CommandResult;

pub fn boot(udid: &str) -> CommandResult {
    let output = Command::new("xcrun")
        .args(["simctl", "boot", udid])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to boot simulator: {}", stderr).into());
    }

    println!("Simulator booted: {}", udid);
    Ok(())
}
```

### ハイブリッド実装例

```rust
pub async fn grant_permission(
    udid: &str,
    bundle: &str,
    permission: &str
) -> CommandResult {
    // まずgRPCを試す
    let grpc_result = with_client(Some(udid), |mut client| async move {
        client.approve(bundle, permission).await
    }).await;

    match grpc_result {
        Ok(_) => {
            println!("Permission granted via gRPC");
            Ok(())
        }
        Err(e) if is_schema_error(&e) => {
            // SQLiteエラーの場合はsimctlにfallback
            eprintln!("gRPC failed (schema error), trying simctl...");
            simctl_grant_permission(udid, bundle, permission)
        }
        Err(e) => Err(e),
    }
}

fn simctl_grant_permission(udid: &str, bundle: &str, perm: &str) -> CommandResult {
    let output = Command::new("xcrun")
        .args(["simctl", "privacy", udid, "grant", perm, bundle])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("simctl privacy grant failed: {}", stderr).into());
    }

    println!("Permission granted via simctl");
    Ok(())
}
```

## 関連ファイル

- **proto/idb.proto**: gRPC RPC定義
- **src/helpers/client.rs**: with_client実装
- **crates/platform-ios/src/grpc/**: idb gRPC実装
- **crates/platform-ios/src/simctl/management.rs**: xcrun simctl wrapper
- **docs/KNOWN_ISSUES.md**: 既知の問題（権限管理のSQLiteエラーなど）

## 判断支援ツール

新機能追加時は以下のスクリプトで判断支援:

```bash
./scripts/platform-check.sh <feature-name>
```

protoファイルを検索し、推奨実装を提示します。

---

**まとめ**: proto/idb.protoにRPC定義があれば idb gRPC、なければ xcrun simctl。両方で可能な場合はハイブリッド実装を検討。
