 ## 技術スタック

 - **Rust 2021**: メインアプリケーション
 - **tokio 1.49**: 非同期ランタイム
 - **tonic 0.12 + prost 0.13**: gRPC クライアント/Protocol Buffers
 - **clap 4.5**: CLI フレームワーク (derive)
 - **idb_companion**: Swift/ObjC ダエモン（外部プロセス）
 - **Python idb**: 参照実装（サブモジュール）

 ## 開発コマンド

 ### アーキテクチャ
```txt
 レイヤー構造とデータフロー

 ┌─────────────────────────────────────────────────────────────┐
 │  CLI Layer (src/cli/)                                        │
 │  - コマンド引数解析 (clap)                                  │
 │  - UDID 指定 → CompanionResolver へ                         │
 │  - 出力フォーマット (JSON/Human)                            │
 └─────────────────────────────────────────────────────────────┘
               ↓ (udid: Option<&str>)
 ┌─────────────────────────────────────────────────────────────┐
 │  Companion Resolution (src/platform/ios/companion/)          │
 │  ┌─────────────────────────────────────────────────────────┐│
 │  │ 1. CompanionState                                       ││
 │  │    - /tmp/idb/state から companion 情報読み込み        ││
 │  │    - JSON: [{udid, path/host/port, pid}]               ││
 │  │ 2. CompanionResolver                                    ││
 │  │    - UDID 指定時: state から該当 companion 検索        ││
 │  │    - UDID 未指定: 単一 companion を自動選択            ││
 │  │    - companion 不在: CompanionSpawner で自動起動       ││
 │  │ 3. 解決結果: ResolvedCompanion {address, udid}         ││
 │  └─────────────────────────────────────────────────────────┘│
 └─────────────────────────────────────────────────────────────┘
               ↓ (Address: DomainSocket | TCP)
 ┌─────────────────────────────────────────────────────────────┐
 │  gRPC Client (src/platform/ios/grpc/client.rs)              │
 │  ┌─────────────────────────────────────────────────────────┐│
 │  │ IdbClient                                               ││
 │  │ - connect_uds(path): Unix Domain Socket 接続           ││
 │  │ - connect_tcp(host, port): TCP 接続                    ││
 │  │ - CompanionServiceClient<Channel> をラップ            ││
 │  └─────────────────────────────────────────────────────────┘│
 │                                                              │
 │  カテゴリ別メソッド (src/platform/ios/grpc/):               │
 │  - app.rs: launch, terminate, list_apps                     │
 │  - file.rs: ls, mkdir, push, pull                           │
 │  - hid.rs: tap, swipe, button, text                         │
 │  - target.rs: describe, list                                │
 │  - test.rs: xctest_run, xctest_list                         │
 └─────────────────────────────────────────────────────────────┘
               ↓ (gRPC Request/Response または Stream)
 ┌─────────────────────────────────────────────────────────────┐
 │  Proto Layer (tonic + prost)                                 │
 │  - build.rs で proto/idb.proto からコード生成              │
 │  - Unix Domain Socket トランスポート (tower service_fn)    │
 │  - ストリーミング RPC 対応                                  │
 └─────────────────────────────────────────────────────────────┘
               ↓ (Unix Domain Socket / TCP)
 ┌─────────────────────────────────────────────────────────────┐
 │  idb_companion (Swift/ObjC 外部プロセス)                    │
 │  - iOS Simulator/Device との通信                            │
 │  - TCC データベース操作 (パーミッション)                   │
 │  - Framebuffer 読み取り (スクリーンショット)               │
 └─────────────────────────────────────────────────────────────┘
 ```

## コマンド構造

agent-mobile CLI は以下の 5 つのメインコマンドで構成されています：

### 1. hid - HID 操作
タッチジェスチャー、キーボード入力、ハードウェアボタン操作を実行します。

**タッチジェスチャー:**
```bash
agent-mobile hid tap 100 200
agent-mobile hid tap center
agent-mobile hid swipe up
agent-mobile hid swipe 100,200,300,400
agent-mobile hid scroll down
agent-mobile hid long-press 150 250
```

**キーボード入力:**
```bash
agent-mobile hid text "Hello World"
agent-mobile hid key enter
agent-mobile hid key backspace
agent-mobile hid clear
```

**ハードウェアボタン:**
```bash
agent-mobile hid button home
agent-mobile hid button lock
agent-mobile hid button volume-up
```

### 2. app - アプリケーション管理
アプリの起動、終了、インストール、パーミッション管理を実行します。

**基本操作:**
```bash
agent-mobile app launch com.example.app
agent-mobile app terminate com.example.app
agent-mobile app install app.ipa
agent-mobile app uninstall com.example.app
agent-mobile app list
```

**パーミッション管理:**
```bash
# カメラ権限を付与
agent-mobile app grant camera --bundle com.example.app

# 位置情報権限を取り消し
agent-mobile app revoke location --bundle com.example.app

# 連絡先権限をリセット
agent-mobile app reset contacts --bundle com.example.app
```

**利用可能なパーミッション (iOS):**
- `camera`, `photos`, `contacts`, `location`, `microphone`
- `calendar`, `reminders`, `siri`, `health`, `homekit`
- `motion`, `speech-recognition`, `faceid`

**利用可能なパーミッション (Android):**
- `camera`, `location`, `contacts`, `calendar`, `storage`
- `microphone`, `phone`, `sms`

### 3. device - デバイス管理
デバイスのリスト表示、起動、停止、クリップボード操作を実行します。

**基本操作:**
```bash
agent-mobile device list
agent-mobile device boot "iPhone 15"
agent-mobile device shutdown <udid>
```

**クリップボード操作 (iOS のみ):**
```bash
# クリップボードにコピー
agent-mobile device pbcopy "Hello World"

# クリップボードから貼り付け
agent-mobile device pbpaste
```

**注:** `pbcopy` / `pbpaste` は macOS の同名コマンドとの親和性を考慮した命名です。

### 4. idb - iOS 専用高度機能
idb の全 59 サブコマンドをサポートします。スクリーンショット、ログ取得、
デバッグサーバー、xctest 実行など、iOS 開発に必要な高度な機能を提供します。

```bash
agent-mobile idb screenshot output.png
agent-mobile idb list-targets
agent-mobile idb accessibility describe-all
agent-mobile idb log
agent-mobile idb xctest-run <bundle-id>
```

## AI エージェント向け推奨パターン

### HID コマンド統合 (2026-01)

**以前のコマンド (非推奨):**
```bash
# ❌ 旧コマンド (削除されました)
agent-mobile gesture tap 100 200
agent-mobile gesture swipe up
agent-mobile keyboard text "Hello World"
agent-mobile keyboard key enter
```

**新しいコマンド:**
```bash
# ✅ 新コマンド
agent-mobile hid tap 100 200
agent-mobile hid swipe up
agent-mobile hid text "Hello World"
agent-mobile hid key enter
```

### パーミッション付与

**以前のコマンド (非推奨):**
```bash
# ❌ 旧コマンド (削除されました)
agent-mobile privacy grant camera --bundle com.example.app
```

**新しいコマンド:**
```bash
# ✅ 新コマンド
agent-mobile app grant camera --bundle com.example.app
```

### クリップボード操作

**以前のコマンド (非推奨):**
```bash
# ❌ 旧コマンド (削除されました)
agent-mobile clipboard copy "Hello"
agent-mobile clipboard paste
```

**新しいコマンド:**
```bash
# ✅ 新コマンド
agent-mobile device pbcopy "Hello"
agent-mobile device pbpaste
```

### よくあるタスク

**アプリのテストシナリオ:**
```bash
# 1. シミュレータ起動
agent-mobile device boot "iPhone 15"

# 2. アプリインストール
agent-mobile app install MyApp.ipa

# 3. 必要な権限を付与
agent-mobile app grant camera --bundle com.example.myapp
agent-mobile app grant location --bundle com.example.myapp

# 4. アプリ起動
agent-mobile app launch com.example.myapp

# 5. UI 操作
agent-mobile hid text "test@example.com"
agent-mobile hid tap 200 400

# 6. スクリーンショット取得
agent-mobile idb screenshot test-result.png
```

**クリーンアップ:**
```bash
# アプリ終了
agent-mobile app terminate com.example.myapp

# アプリアンインストール
agent-mobile app uninstall com.example.myapp

# シミュレータ停止
agent-mobile device shutdown <udid>
```
