# xcrun simctl vs idb gRPC: 設計ガイドライン

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

## 機能別の推奨

### xcrun simctl を使う

| 機能 | コマンド | 理由 |
|------|---------|------|
| シミュレータ起動 | `xcrun simctl boot <udid>` | proto に boot RPC なし |
| シミュレータ停止 | `xcrun simctl shutdown <udid>` | proto に shutdown RPC なし |
| シミュレータ作成/削除 | `xcrun simctl create/delete` | ライフサイクル管理 |
| **クリップボード** | `xcrun simctl pbcopy/pbpaste <udid>` | **proto に API なし** |

### idb gRPC を使う

| 機能 | proto RPC | 理由 |
|------|----------|------|
| HID 入力 (tap/swipe/text) | `HIDEvent` | 精密な制御、ストリーミング |
| スクリーンショット | `ScreenshotRequest` | Framebuffer 直接アクセス |
| ファイル操作 | `FileContainer` | 統一インターフェース |
| アプリ操作 | `InstallRequest`, `LaunchRequest` | 進捗ストリーミング |
| ログ取得 | `LogRequest` | リアルタイムストリーミング |

## 各アプローチの特徴

### xcrun simctl
- **利点**: Apple 公式、安定、追加依存なし、シンプル
- **欠点**: シミュレータ限定、実機非対応、ストリーミング不可

### idb gRPC
- **利点**: 型安全、ストリーミング対応、実機対応、高度な機能
- **欠点**: idb_companion 必要、接続オーバーヘッド、一部機能なし

## 現在の実装状況

```
src/platform/ios/
├── simctl/management.rs    # boot, shutdown, create, delete, erase (xcrun)
├── grpc/                   # HID, file, screenshot, app (idb gRPC)
└── companion/              # companion 管理
```

## 今後の拡張について

新しい `agent-mobile <feature>` を追加する際:

1. まず `proto/idb.proto` を確認
2. RPC 定義があれば → gRPC で実装
3. なければ → `xcrun simctl` で実装
4. CLI は `src/idb/` に追加

### クリップボード実装例（必要な場合）

```rust
// src/platform/ios/simctl/management.rs に追加
pub fn pbcopy(udid: &str, text: &str) -> Result<()> {
    Command::new("xcrun")
        .args(["simctl", "pbcopy", udid])
        .stdin(Stdio::piped())
        .spawn()?.stdin.unwrap().write_all(text.as_bytes())?;
    Ok(())
}

pub fn pbpaste(udid: &str) -> Result<String> {
    let output = Command::new("xcrun")
        .args(["simctl", "pbpaste", udid])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

---

## 重複機能がある場合の判断基準

両方で実装可能な機能の場合、以下の優先順位で選択:

### 1. idb gRPC を優先するケース

| 理由 | 例 |
|------|-----|
| ストリーミングが必要 | アプリインストール（進捗表示）|
| 型安全が重要 | アプリ起動（エラーハンドリング）|
| 実機対応が必要 | スクリーンショット |
| 高度な機能が必要 | アクセシビリティ情報 |

### 2. simctl をフォールバックとして使うケース

| 状況 | 例 |
|------|-----|
| idb gRPC でエラー | 権限管理（SQLite スキーマエラー）|
| idb_companion 不可時 | 起動前の操作 |

### 3. simctl のみを使うケース

| 理由 | 例 |
|------|-----|
| proto に API がない | シミュレータ作成/クローン |
| ライフサイクル管理 | boot/shutdown（現状） |

## 機能別の実装状況

| 機能 | idb gRPC | xcrun simctl | 現在の選択 | 理由 |
|------|---------|-------------|-----------|------|
| アクセシビリティ | ✓ | ✗ | **gRPC** | simctl では不可 |
| アプリ起動/終了 | ✓ | ✓ | **gRPC** | ストリーミング対応 |
| アプリインストール | ✓ | ✓ | **gRPC** | 進捗表示 |
| スクリーンショット | ✓ | ✓ | **gRPC** | Framebuffer 直接 |
| 権限管理 | ✓ | ✓ | **gRPC** (+ simctl fallback) | エラー時切替 |
| 通知送信 | ✓ | ✓ | **gRPC** | 統一 API |
| シミュレータ起動 | △ | ✓ | **simctl** | proto に boot RPC なし |
| シミュレータ作成 | ✗ | ✓ | **simctl** | gRPC 未対応 |
| クリップボード | ✗ | ✓ | **simctl** | gRPC 未対応 |
| HID 入力 | ✓ | ✗ | **gRPC** | simctl では不可 |

## ハイブリッド実装パターン

```rust
// 推奨パターン: フォールバック対応
pub async fn grant_permission(bundle: &str, perm: Permission) -> Result<()> {
    // 1. まず idb gRPC を試す
    match idb_client.approve(bundle, perm).await {
        Ok(_) => Ok(()),
        Err(e) if e.is_schema_error() => {
            // 2. 失敗時は simctl にフォールバック
            simctl::privacy_grant(udid, perm, bundle)
        }
        Err(e) => Err(e),
    }
}
```

## 関連ファイル

- `src/platform/ios/simctl/management.rs` - xcrun simctl ラッパー
- `src/platform/ios/grpc/client.rs` - gRPC クライアント
- `src/platform/ios/grpc/device.rs` - アクセシビリティ実装
- `proto/idb.proto` - 利用可能な gRPC RPC 定義
- `docs/KNOWN_ISSUES.md` - 既知の問題（権限管理エラーなど）
