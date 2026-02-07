# Android Console - リアルタイムログストリーミング

> `agent-mobile console` コマンドの Android 実装ガイド

## 📋 目次

- [概要](#概要)
- [前提条件](#前提条件)
- [基本的な使い方](#基本的な使い方)
- [実装アーキテクチャ](#実装アーキテクチャ)
- [トラブルシューティング](#トラブルシューティング)
- [技術詳細](#技術詳細)

---

## 概要

`agent-mobile console` コマンドは、Android デバイス/エミュレーターのログ (logcat) をリアルタイムでストリーミング表示します。

### 特徴

- ✅ **ADB native protocol**: `adb logcat` CLI を使わず、TCP :5037 で直接通信
- ✅ **非同期ストリーミング**: tokio ベースの高速処理
- ✅ **Ctrl+C 対応**: 即座にストリーミング停止
- ✅ **デバイス自動選択**: UDID 省略で接続中のデバイスを自動検出

---

## 前提条件

### 1. ADB サーバーの起動

**重要**: agent-mobile は ADB サーバーを自動起動しません。事前に起動が必要です。

```bash
# ADB サーバーを起動
adb start-server

# 起動確認
adb devices
# List of devices attached
# emulator-5554	device
```

**初回セットアップ**:
```bash
# Android SDK のインストール (Android Studio 推奨)
# https://developer.android.com/studio

# PATH 設定 (~/.zshrc または ~/.bashrc に追加)
export ANDROID_HOME=$HOME/Library/Android/sdk
export PATH=$PATH:$ANDROID_HOME/platform-tools

# 確認
adb version
# Android Debug Bridge version 1.0.41
```

### 2. デバイス/エミュレーターの接続

```bash
# エミュレーター起動 (例)
emulator -avd "Pixel_6_API_34" &

# または、既存の起動済みエミュレーターを確認
adb devices
```

---

## 基本的な使い方

### 1. デバイス UDID を指定

```bash
agent-mobile console --udid emulator-5554
```

**出力例**:
```text
01-15 10:23:45.123  1234  5678 I ActivityManager: Start proc com.example.app
01-15 10:23:45.234  1234  5678 D NetworkSecurityConfig: No Network Security Config specified
01-15 10:23:45.345  1234  5678 I chatty: uid=10123(com.example.app) identical 3 lines
...
```

**停止**: `Ctrl+C` で即座に終了

---

### 2. デバイス自動選択

UDID を省略すると、接続中のデバイスを自動選択します。

```bash
agent-mobile console --platform android
```

**動作**:
- 複数デバイスが接続されている場合、最初に見つかったデバイスを使用
- ADB の `host:transport-any` プロトコルを利用

---

### 3. 開発時のビルド & 実行

```bash
# デバッグビルド
cargo build

# 実行
cargo run -- console --udid emulator-5554

# リリースビルド
cargo build --release
./target/release/agent-mobile console --udid emulator-5554
```

---

## 実装アーキテクチャ

### 処理フロー

```text
CLI (src/main.rs)
    ↓
gateway/console.rs (stream_android_console)
    ↓
gateway/api/android.rs (AndroidDevice::stream_logs)
    ↓
platform-android/adb/logcat.rs (LogcatStream)
    ↓
ADB wire protocol (TCP 127.0.0.1:5037)
    ↓
ADB サーバー → Android デバイス
```

### コア実装: LogcatStream

**ファイル**: `crates/platform-android/src/adb/logcat.rs`

```rust
pub struct LogcatStream {
    lines: Lines<BufReader<TcpStream>>,
}

impl LogcatStream {
    pub async fn open(serial: Option<&str>) -> Result<Self> {
        // 1. TCP 接続 (127.0.0.1:5037)
        let mut stream = TcpStream::connect("127.0.0.1:5037").await?;

        // 2. デバイス選択
        send_command(&mut stream, "host:transport:emulator-5554").await?;
        read_okay(&mut stream).await?;

        // 3. logcat 開始
        send_command(&mut stream, "shell:logcat").await?;
        read_okay(&mut stream).await?;

        // 4. 行指向リーダーでラップ
        Ok(Self { lines: BufReader::new(stream).lines() })
    }

    pub async fn next_line(&mut self) -> Result<Option<String>> {
        self.lines.next_line().await
    }
}
```

### ADB wire protocol の詳細

**コマンド形式**: `{4桁16進数長さ}{コマンド本文}`

```rust
// 例: "host:transport-any" (18文字)
let msg = format!("{:04X}{}", "host:transport-any".len(), "host:transport-any");
// 結果: "0012host:transport-any"
```

**応答形式**:
- 成功: `OKAY` (4バイト)
- 失敗: `FAIL{4桁16進数長さ}{エラーメッセージ}`

---

## トラブルシューティング

### エラー: "ADB server not reachable at 127.0.0.1:5037"

**原因**: ADB サーバーが起動していない

**解決方法**:
```bash
# サーバーを起動
adb start-server

# 確認
adb devices
```

---

### エラー: "device not found"

**原因**: 指定した UDID のデバイスが存在しない

**解決方法**:
```bash
# 接続中のデバイスを確認
adb devices
# List of devices attached
# emulator-5554	device

# 正しい UDID を使用
agent-mobile console --udid emulator-5554
```

---

### ポート競合 (5037 が使用中)

**確認**:
```bash
lsof -i :5037
# COMMAND   PID   USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
# adb      1234  user    3u  IPv4  0x...      0t0  TCP localhost:5037 (LISTEN)
```

**解決方法**:
```bash
# ADB サーバーを再起動
adb kill-server
adb start-server
```

---

### ログが表示されない

**原因 1**: logcat バッファが空

```bash
# テストログを生成
adb shell log -t TestTag "Test message"

# agent-mobile で確認
agent-mobile console --udid emulator-5554
```

**原因 2**: ログレベルフィルタ

現在の実装はフィルタなし (全ログ表示)。特定のタグのみ表示したい場合は、将来的な拡張が必要です。

---

## 技術詳細

### 依存クレート

**`crates/platform-android/Cargo.toml`**:
```toml
# ADB protocol client (TCP :5037 direct communication)
adb_client = "2.0"

# Async runtime
tokio = { version = "1.49.0", features = ["net", "io-util"] }
```

### 非同期ストリーミング制御

**`gateway/console.rs:98-116`**:
```rust
loop {
    tokio::select! {
        // Ctrl+C シグナル検知
        _ = stop_rx.changed() => {
            if *stop_rx.borrow() {
                stream.stop().await?;
                break;
            }
        }
        // ログ行の読み取り
        line = stream.next_line() => {
            match line? {
                Some(line) => {
                    writeln!(writer, "{}", line)?;
                    writer.flush()?;  // 即座に標準出力へ
                }
                None => break,
            }
        }
    }
}
```

### パフォーマンス最適化

| 最適化 | 詳細 |
|--------|------|
| **CLI レス** | `adb logcat` プロセス起動なし、直接 TCP 通信 |
| **バッファリング** | `BufReader` で syscall 回数削減 |
| **非同期 I/O** | tokio の `TcpStream` でノンブロッキング処理 |
| **即時フラッシュ** | `writer.flush()` で遅延なし表示 |

---

## 開発者向け情報

### テスト

**ユニットテスト**:
```bash
cargo test --package agent-mobile-platform-android --lib adb::logcat::tests
```

**実機テスト** (E2E):
```bash
# エミュレーター起動 + console 実行
/mobile-e2e android
```

### 関連ファイル

| ファイル | 役割 |
|---------|------|
| `crates/gateway/src/console.rs` | トップレベル制御、プラットフォーム分岐 |
| `crates/gateway/src/api/android.rs` | AndroidDevice ラッパー |
| `crates/platform-android/src/adb/logcat.rs` | **コア実装** (ADB protocol) |
| `crates/platform-android/src/adb/commands.rs` | 公開 API (is_adb_available など) |
| `crates/platform-android/src/adb/connection.rs` | AdbConnection (他機能用) |

---

## 参考資料

### 公式ドキュメント

- [Android ADB Protocol](https://android.googlesource.com/platform/packages/modules/adb/+/refs/heads/master/protocol.txt)
- [Android Debug Bridge (adb)](https://developer.android.com/tools/adb)
- [logcat コマンドラインツール](https://developer.android.com/tools/logcat)

### 関連リソース

- [adb_client crate](https://docs.rs/adb_client/) - ADB プロトコルクライアント
- [tokio](https://tokio.rs/) - 非同期ランタイム
- [CLAUDE.md](../CLAUDE.md) - 開発ワークフロー (日本語)

---

## まとめ

### チェックリスト

実行前の確認項目:

- [ ] ADB サーバーが起動している (`adb start-server`)
- [ ] デバイス/エミュレーターが接続されている (`adb devices`)
- [ ] UDID が正しい (またはデバイスが1台のみ接続)

### コマンド早見表

```bash
# セットアップ
adb start-server
adb devices

# ログストリーミング開始
agent-mobile console --udid emulator-5554

# 停止
Ctrl+C

# トラブルシューティング
adb kill-server && adb start-server
lsof -i :5037
```