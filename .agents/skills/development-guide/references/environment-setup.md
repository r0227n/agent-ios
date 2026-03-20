# agent-mobile 環境セットアップガイド

agent-mobile CLI開発における環境セットアップの完全ガイド。初回セットアップ、トラブルシューティング、プラットフォーム別要件を網羅します。

## 目次

- [概要](#概要)
- [iOS環境セットアップ](#ios環境セットアップ)
- [Android環境セットアップ](#android環境セットアップ)
- [クイックセットアップスクリプト](#クイックセットアップスクリプト)
- [トラブルシューティング](#トラブルシューティング)
- [開発ワークフローへの統合](#開発ワークフローへの統合)

## 概要

### 前提条件

agent-mobile CLI開発を開始する前に、以下の環境が必要です:

**必須:**
- macOS (iOS開発の場合)
- Rust 2021以降
- cargo (Rustに付属)

**iOS開発:**
- Xcode (Command Line Tools含む)

**Android開発:**
- Android SDK (adb, emulator)
- Java Development Kit (JDK)

### セットアップのタイミング

以下の場合にセットアップを実行してください:

- **初回セットアップ**: agent-mobile開発を初めて行う場合
- **システムアップデート後**: Xcode、Android Studioなどを更新した場合
- **プラットフォーム切り替え**: iOS ↔ Android開発を切り替える場合
- **デバイス検出エラー**: `agent-mobile device list` でデバイスが検出されない場合

## iOS環境セットアップ

### 必要なツール

#### 1. Xcode Command Line Tools

**確認方法:**
```bash
xcrun simctl list devices
```

**インストール:**
```bash
xcode-select --install
```

**トラブルシューティング:**
- エラー: `xcrun: error: unable to find utility "simctl"`
  - 原因: Command Line Toolsが未インストール
  - 解決: 上記のインストールコマンドを実行

- エラー: `xcode-select: error: tool 'xcodebuild' requires Xcode`
  - 原因: Xcode本体が必要
  - 解決: App StoreからXcodeをインストール

### シミュレータ管理

#### シミュレータ一覧確認

```bash
xcrun simctl list devices
```

**出力例:**
```
-- iOS 17.0 --
    iPhone 15 (12345678-1234-1234-1234-123456789012) (Shutdown)
    iPhone 15 Pro (87654321-4321-4321-4321-210987654321) (Booted)
```

#### シミュレータ起動

```bash
# UDID指定
xcrun simctl boot <udid>

# Simulator.appも開く
open -a Simulator
```

#### シミュレータ作成

```bash
# 最新iOS向けiPhone 15を作成
xcrun simctl create "iPhone 15" "iPhone 15" "iOS17.0"
```

### iOS環境検証

```bash
# 1. simctlが動作するか
xcrun simctl list devices | head -20

# 2. シミュレータが起動しているか
xcrun simctl list devices | grep Booted

# 3. agent-mobileがデバイスを認識するか
agent-mobile device list
```

## Android環境セットアップ

### 必要なツール

#### 1. Android SDK

**インストール方法:**

**Option 1: Android Studioから (推奨):**
1. [Android Studio](https://developer.android.com/studio)をダウンロード
2. SDK Managerを開く (Tools → SDK Manager)
3. 必要なコンポーネントをインストール:
   - Android SDK Platform-Tools
   - Android SDK Build-Tools
   - Android Emulator
   - System Images (例: Google APIs Intel x86 Atom System Image)

**Option 2: Command Line Toolsのみ:**
```bash
# ダウンロード
wget https://dl.google.com/android/repository/commandlinetools-mac-9477386_latest.zip

# 展開
unzip commandlinetools-mac-9477386_latest.zip -d ~/android-sdk
cd ~/android-sdk/cmdline-tools
mkdir latest
mv bin lib NOTICE.txt source.properties latest/

# SDK Managerでツールをインストール
./latest/bin/sdkmanager "platform-tools" "emulator" "build-tools;33.0.0"
```

**環境変数設定:**
```bash
# ~/.zshrc または ~/.bashrc に追加
export ANDROID_HOME=$HOME/Library/Android/sdk
export PATH=$PATH:$ANDROID_HOME/platform-tools
export PATH=$PATH:$ANDROID_HOME/emulator
```

**適用:**
```bash
source ~/.zshrc  # または source ~/.bashrc
```

#### 2. adb (Android Debug Bridge)

**確認方法:**
```bash
which adb
adb version
```

**トラブルシューティング:**
- エラー: `adb: command not found`
  - 原因: Android SDKが未インストール、またはPATHが通っていない
  - 解決: 環境変数 `$ANDROID_HOME/platform-tools` をPATHに追加

- エラー: `adb server version doesn't match this client`
  - 原因: 複数のadbバージョンが混在
  - 解決: `adb kill-server && adb start-server`

#### 3. emulator

**確認方法:**
```bash
which emulator
emulator -version
```

**トラブルシューティング:**
- エラー: `emulator: command not found`
  - 原因: emulatorがPATHに含まれていない
  - 解決: `$ANDROID_HOME/emulator` をPATHに追加（`platform-tools`より**前**に配置）

### AVD (Android Virtual Device) 管理

#### AVD一覧確認

```bash
emulator -list-avds
```

#### AVD作成 (Android Studioから推奨)

1. Android Studioを開く
2. Tools → Device Manager
3. "Create Device" をクリック
4. デバイスタイプを選択 (例: Pixel 6)
5. System Imageを選択 (例: Android 13 - API 33)
6. AVD名を設定 (例: Pixel_6_API_33)
7. "Finish" をクリック

#### AVD作成 (コマンドライン)

```bash
# 利用可能なSystem Imageを確認
sdkmanager --list | grep system-images

# System Imageをインストール
sdkmanager "system-images;android-33;google_apis;x86_64"

# AVDを作成
avdmanager create avd -n Pixel_6_API_33 \
  -k "system-images;android-33;google_apis;x86_64" \
  -d "pixel_6"
```

#### エミュレータ起動

```bash
# AVD名指定
emulator -avd Pixel_6_API_33

# バックグラウンド起動
nohup emulator -avd Pixel_6_API_33 >/dev/null 2>&1 &

# スナップショット無効化（起動高速化）
emulator -avd Pixel_6_API_33 -no-snapshot-load
```

### adb server管理

```bash
# サーバー状態確認
adb get-state

# サーバー起動
adb start-server

# サーバー再起動
adb kill-server && adb start-server

# 接続デバイス一覧
adb devices
```

**出力例:**
```
List of devices attached
emulator-5554   device
```

### Android環境検証

```bash
# 1. adbが動作するか
adb version

# 2. emulatorがインストールされているか
emulator -version

# 3. AVDが作成されているか
emulator -list-avds

# 4. エミュレータが起動しているか
adb devices

# 5. agent-mobileがデバイスを認識するか
agent-mobile device list
```

## クイックセットアップスクリプト

### iOS自動セットアップ

```bash
./.agents/skills/development-guide/scripts/setup-ios.sh
```

**実行内容:**
1. ✅ Xcode Command Line Tools確認
2. ✅ シミュレータ起動（未起動の場合）
3. ✅ agent-mobile接続確認

**出力例:**
```
=== iOS Device Setup ===

>>> Checking Xcode installation...
  [OK] xcrun simctl available

>>> Checking simulator status...
  [OK] Simulator already booted
  [INFO] iPhone 15 (12345678-1234-1234-1234-123456789012) (Booted)

>>> Verifying connection...
  [OK] agent-mobile can communicate with device

=== Setup Complete ===
UDID: 12345678-1234-1234-1234-123456789012
Status: Ready for testing

Next steps:
  agent-mobile device list    # Verify device
  agent-mobile snapshot       # Get UI snapshot
  agent-mobile <command>      # Run the changed command directly
```

### Android自動セットアップ

```bash
./.agents/skills/development-guide/scripts/setup-android.sh
```

**実行内容:**
1. ✅ Android SDK (adb, emulator)確認
2. ✅ adb server起動
3. ✅ エミュレータ起動（未起動の場合）
4. ✅ agent-mobile接続確認

**出力例:**
```
=== Android Device Setup ===

>>> Checking Android SDK...
  [OK] adb found at /Users/user/Library/Android/sdk/platform-tools/adb
  [OK] emulator found at /Users/user/Library/Android/sdk/emulator/emulator

>>> Checking adb server...
  [OK] adb server running

>>> Checking emulator status...
  [INFO] Available AVDs:
    - Pixel_6_API_33
  [OK] Emulator already running: emulator-5554

>>> Verifying connection...
  [OK] agent-mobile can communicate with device

=== Setup Complete ===
Device: emulator-5554
Status: Ready for testing

Next steps:
  agent-mobile device list    # Verify device
  adb shell                   # Direct shell access
  agent-mobile <command>      # Run the changed command directly
```

## トラブルシューティング

### iOS関連

#### シミュレータが起動しない

**症状:**
```
An error was encountered processing the command (domain=NSPOSIXErrorDomain, code=60)
```

**原因:** ディスク容量不足、破損したシミュレータ

**解決:**
```bash
# シミュレータをリセット
xcrun simctl erase <udid>

# または新しいシミュレータを作成
xcrun simctl create "iPhone 15 Clean" "iPhone 15" "iOS17.0"
```

#### agent-mobileがデバイスを検出しない

**症状:**
```
No iOS devices detected
```

**デバッグ手順:**
```bash
# 1. シミュレータが起動しているか
xcrun simctl list devices | grep Booted

# 2. agent-mobileをデバッグモードで実行
RUST_LOG=debug agent-mobile device list
```

### Android関連

#### adbがデバイスを検出しない

**症状:**
```
List of devices attached
# 空っぽ
```

**デバッグ手順:**
```bash
# 1. エミュレータが起動しているか
ps aux | grep emulator

# 2. adb serverを再起動
adb kill-server && adb start-server

# 3. デバイス再検出
adb devices

# 4. ログ確認
adb logcat | grep -i error
```

#### エミュレータが起動に失敗する

**症状:**
```
emulator: ERROR: x86_64 emulation currently requires hardware acceleration!
```

**原因:** ハードウェアアクセラレーションが無効

**解決 (Intel Mac):**
```bash
# Intel HAXM確認
kextstat | grep intel

# 未インストールの場合
# Android Studio → Tools → SDK Manager → SDK Tools → Intel x86 Emulator Accelerator (HAXM)
```

**解決 (Apple Silicon Mac):**
```bash
# ARM64イメージを使用
sdkmanager "system-images;android-33;google_apis;arm64-v8a"

# AVDを再作成
avdmanager create avd -n Pixel_6_ARM \
  -k "system-images;android-33;google_apis;arm64-v8a" \
  -d "pixel_6"
```

#### AVDが見つからない

**症状:**
```
PANIC: Cannot find AVD system path. Please define ANDROID_SDK_ROOT
```

**原因:** ANDROID_SDK_ROOT環境変数が未設定

**解決:**
```bash
# ~/.zshrc または ~/.bashrc に追加
export ANDROID_SDK_ROOT=$HOME/Library/Android/sdk

# 適用
source ~/.zshrc
```

### 共通問題

#### agent-mobileコマンドが見つからない

**症状:**
```
agent-mobile: command not found
```

**原因:** バイナリが未ビルド、またはPATHが通っていない

**解決:**
```bash
# プロジェクトルートで
cd /Users/r0227n/Dev/agent-mobile

# ビルド
cargo build --release

# PATHに追加（一時的）
export PATH=$PATH:$(pwd)/target/release

# または絶対パスで実行
./target/release/agent-mobile device list
```

#### 権限エラー

**症状:**
```
Permission denied
```

**iOS (simctl):**
```bash
# セットアップスクリプトに実行権限付与
chmod +x .agents/skills/development-guide/scripts/setup-ios.sh
```

**Android (adb):**
```bash
# adbに実行権限付与
chmod +x $ANDROID_HOME/platform-tools/adb
```

## 開発ワークフローへの統合

### Phase 0: 環境セットアップ（必須前提条件）

agent-mobile開発の6ステップワークフロー:

```
Phase 0: 環境セットアップ（このガイド）
   ↓
Phase 1: 設計（プラットフォーム判断、引数設計）
   ↓
Phase 2: 実装（コマンド作成、iOS/Android実装）
   ↓
Phase 3: ユニットテスト（ロジック検証）
   ↓
Phase 4: 統合テスト（CLI動作検証）
   ↓
Phase 5: 実機動作確認（視覚的検証、必須!）
   ↓
Phase 6: コミット
```

### 環境確認コマンド

開発開始前に実行:

```bash
# iOS環境
./.agents/skills/development-guide/scripts/setup-ios.sh

# Android環境
./.agents/skills/development-guide/scripts/setup-android.sh

# または、run-tests.shが自動チェック
./.agents/skills/development-guide/scripts/run-tests.sh
```

### 継続的な環境メンテナンス

#### 定期確認（推奨: 週1回）

```bash
# iOS
xcrun simctl list devices | grep Booted

# Android
sdkmanager --update
adb version
```

#### システムアップデート後

```bash
# Xcodeアップデート後
xcode-select --install
xcrun simctl list runtimes

# Android Studioアップデート後
sdkmanager --list
emulator -version
```

## まとめ

### クイックスタート

**iOS開発:**
```bash
# 1. 環境セットアップ
./.agents/skills/development-guide/scripts/setup-ios.sh

# 2. 確認
agent-mobile device list
```

**Android開発:**
```bash
# 1. 環境セットアップ
./.agents/skills/development-guide/scripts/setup-android.sh

# 2. 確認
agent-mobile device list
```

### トラブル時の対処

1. **スクリプトでの自動セットアップを試す**
2. **このガイドのトラブルシューティングセクションを参照**
3. **ログ確認**: `RUST_LOG=debug agent-mobile <command>`
4. **環境の再構築**: シミュレータ/エミュレータの再作成

### 関連ドキュメント

- **SKILL.md**: 開発ワークフロー全体
- **testing-guide.md**: テスト戦略（Phase 3-5）
- **README.md**: スキル全体のガイド
- **CLAUDE.md (プロジェクトルート)**: agent-mobile使用方法
