 ## 技術スタック

 - **Rust 2021**: メインアプリケーション
 - **tokio 1.49**: 非同期ランタイム
 - **tonic 0.12 + prost 0.13**: gRPC クライアント/Protocol Buffers
 - **clap 4.5**: CLI フレームワーク (derive)
 - **idb_companion**: Swift/ObjC ダエモン（外部プロセス）
 - **Python idb**: 参照実装（サブモジュール）

 ## 開発ガイド

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

### 開発フロー

`agent-mobile <feature>` の変更時は、以下のサイクルで開発を進めます：

```
実装 → ビルド → ユニットテスト → 統合テスト → 実機動作確認 → コミット
```

**重要**: 実機動作確認は必須ステップです。コードが正しくビルドできても、実際のシミュレータ/デバイスで期待通りに動作することを確認する必要があります。

#### 基本的な開発サイクル

```bash
# 1. 機能実装
# src/ 配下のファイルを編集

# 2. ビルド検証
cargo build --verbose

# 3. ユニットテスト
cargo test --verbose --bins

# 4. 実機動作確認 (必須！)
# → 次のセクション「実機動作確認（必須）」を参照

# 5. 統合テスト (test-setup実行済みの場合)
cargo test --test idb -- --test-threads=1

# 6. コミット
git add .
git commit -m "feat: 変更内容の説明"
```

**ポイント**:
- **実機確認なしでのコミットは禁止**: ビルドが通っても、実際の動作を確認するまでコミットしないでください
- **スクリーンショットで証跡を残す**: 動作確認時は必ずスクリーンショットを取得して、視覚的な証拠を残します
- **Python idbとの比較**: 既存のidb機能を変更する場合、Python版との動作一致を確認します

### 実機動作確認（必須）

#### iOS の動作確認手順

```bash
# 1. テスト環境起動
/device-setup ios

# 2. デバイス確認
agent-mobile device list

# 3. 変更した機能をテスト
agent-mobile <your-command>

# 4. UI状態をスクリーンショットで確認
agent-mobile idb screenshot /tmp/verify-$(date +%Y%m%d-%H%M%S).png

# 5. 自動E2Eテスト実行 (オプション)
/mobile-e2e ios
```

#### 動作確認チェックリスト

各機能変更後、以下を必ず確認してください：

- [ ] コマンドが期待通りの動作をする
- [ ] エラーメッセージが適切に表示される
- [ ] UI操作の結果が視覚的に確認できる
- [ ] スクリーンショットで証跡を保存した
- [ ] Python idbとの動作差異がない (該当する場合)

**例: HID tap コマンドの動作確認**

```bash
# シミュレータ起動
agent-mobile device boot "iPhone 15"

# Safariを起動
agent-mobile app launch com.apple.mobilesafari

# 画面中央をタップ
agent-mobile hid tap center

# 結果をスクリーンショットで確認
agent-mobile idb screenshot /tmp/tap-center-result.png

# 座標指定でタップ
agent-mobile hid tap 100 200

# 結果を確認
agent-mobile idb screenshot /tmp/tap-coords-result.png
```

**例: パーミッション付与の動作確認**

```bash
# カメラ権限を付与
agent-mobile app grant camera --bundle com.apple.mobilesafari

# アプリを再起動して権限を確認
agent-mobile app terminate com.apple.mobilesafari
agent-mobile app launch com.apple.mobilesafari

# スクリーンショットで権限ダイアログを確認
agent-mobile idb screenshot /tmp/camera-permission.png
```

**Python idb との比較確認**

既存機能を変更した場合、Python版との動作一致を確認します：

```bash
# Rust版で実行
agent-mobile idb list-targets

# Python版で実行 (idb_companion/idb ディレクトリで)
poetry run idb list-targets

# 出力を比較して、同じ結果が得られることを確認
```

#### Android の動作確認手順

```bash
# 1. テスト環境起動
/device-setup android

# 2. デバイス確認
agent-mobile device list

# 3. 変更した機能をテスト
agent-mobile <your-command>

# 4. スクリーンショット確認
agent-mobile screenshot /tmp/verify-android-$(date +%Y%m%d-%H%M%S).png

# 5. 自動E2Eテスト実行 (オプション)
/mobile-e2e android
```

**注**: Android版は現在開発中のため、一部機能は未実装の可能性があります。実装状況は `src/platform/android/` ディレクトリを参照してください。

### テスト環境セットアップ

#### 自動セットアップ (推奨)

```bash
# iOS + Android の完全なテスト環境を構築
mise run test-setup

# または個別にセットアップ
/device-setup ios
/device-setup android
```

#### セットアップ内容

`mise run test-setup` は以下を自動実行します：

**iOS環境:**
1. Xcode Command Line Tools の確認
2. idb_companion のビルド (Swift/ObjC)
3. Python idb のインストール (poetry)
4. テストアプリ (TestApp.app) のビルド
5. シミュレータの起動とアプリインストール

**Android環境:**
1. Android SDK のインストール
2. エミュレータの作成と起動
3. テストアプリのインストール

#### 手動セットアップ

自動セットアップが失敗する場合は、以下を手動実行：

```bash
# 1. idb_companion のビルド
cd idb_companion
xcodebuild -project idb_companion.xcodeproj -target idb_companion

# 2. Python idb のインストール
cd idb
poetry install

# 3. テストアプリのビルド
xcodebuild -project TestApp.xcworkspace -scheme TestApp -sdk iphonesimulator

# 4. シミュレータ起動
xcrun simctl boot "iPhone 15"

# 5. アプリインストール
xcrun simctl install booted TestApp.app
```

#### トラブルシューティング

セットアップで問題が発生した場合は、`docs/KNOWN_ISSUES.md` を参照してください。

### CI/CD統合

#### GitHub Actions ワークフロー

`.github/workflows/idb-ci.yml` で3段階の自動検証を実行しています：

**Stage 1: CLI Validation**
- Rust プロジェクトのビルド
- `agent-mobile --help` の実行確認
- 基本的なCLI動作検証

**Stage 2: Unit Tests**
- `cargo test --bins` でユニットテストを実行
- モジュール単位の機能検証

**Stage 3: Integration Tests**
- テスト環境のセットアップ (`mise run test-setup`)
- `cargo test --test idb` で統合テストを実行
- Python idb との動作比較

#### ローカルでCI相当のテスト実行

プルリクエスト前に、ローカルで同じテストを実行することを推奨します：

```bash
# 1. ビルド検証
cargo build --verbose

# 2. ユニットテスト
cargo test --verbose --bins

# 3. 統合テスト (要: mise run test-setup 実行済み)
cargo test --test idb -- --test-threads=1
```

**重要**: `--test-threads=1` は必須です。並列実行すると idb_companion の競合が発生します。

#### テストカバレッジ

統合テストは以下をカバーします：

- **基本コマンド**: list-targets, describe-target
- **HID操作**: tap, swipe, text, button
- **アプリ管理**: launch, terminate, list-apps
- **ファイル操作**: ls, mkdir, push, pull
- **メディア**: screenshot, screen-record
- **パーミッション**: approve, revoke (iOS)

新機能を追加した場合、`tests/idb/` ディレクトリに対応するテストケースを追加してください。

### ベストプラクティス

#### テスト駆動開発 (TDD)

新機能開発時は、以下の順序を推奨します：

```bash
# 1. テストケースを先に書く
# tests/idb/your_feature_test.rs を作成

# 2. 実装する
# src/platform/ios/grpc/your_feature.rs を作成

# 3. テストを実行して確認
cargo test --test idb your_feature -- --test-threads=1

# 4. 実機で動作確認
agent-mobile <your-command>
agent-mobile idb screenshot /tmp/verify.png
```

#### ヘルパー関数の活用

`tests/idb/common/mod.rs` には、テストで使える便利な関数があります：

```rust
// CompanionClient の取得
let client = common::get_client().await?;

// テストアプリの起動
common::launch_test_app(&client).await?;

// スクリーンショットの取得
let screenshot = common::take_screenshot(&client).await?;

// ファイル操作
common::push_file(&client, "/path/to/source", "/path/to/dest").await?;
```

#### 既存パターンの踏襲

新しいコマンドを追加する際は、既存の実装を参考にしてください：

**CLI定義**: `src/cli/idb/mod.rs`
```rust
#[derive(Debug, Subcommand)]
pub enum IdbCommand {
    YourCommand(YourCommandArgs),
}
```

**gRPC実装**: `src/platform/ios/grpc/your_feature.rs`
```rust
impl IdbClient {
    pub async fn your_command(&mut self, args: YourCommandArgs) -> Result<YourCommandResponse> {
        // 実装
    }
}
```

**テスト**: `tests/idb/your_feature_test.rs`
```rust
#[tokio::test]
async fn test_your_command() -> Result<()> {
    let client = common::get_client().await?;
    let result = client.your_command(args).await?;
    assert!(result.is_ok());
    Ok(())
}
```

#### ドキュメント更新

新機能追加時は、以下のドキュメントも更新してください：

- **CLAUDE.md**: AIエージェント向けの使用例
- **README.md**: ユーザー向けクイックスタート
- **docs/ARCHITECTURE.md**: アーキテクチャへの影響
- **proto/idb.proto**: gRPC API の変更 (該当する場合)

#### AIエージェント対応

Claude Code などの AIエージェントが効率的に使えるように、以下を心がけてください：

- **JSON出力対応**: `--json` フラグで JSON 形式の出力をサポート
- **明確なエラーメッセージ**: エラー時に何が問題か、どう解決するかを明示
- **ヘルプの充実**: `--help` で十分な情報を提供
- **冪等性**: 同じコマンドを複数回実行しても安全

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
