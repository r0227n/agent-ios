# Platform Differences: iOS vs Android

agent-mobile における iOS と Android の違いと特性の完全ガイド。

## Overview

agent-mobile は iOS と Android の両方をサポートしていますが、各プラットフォームの技術的な違いにより、実装方法や機能の可用性が異なります。このドキュメントでは、プラットフォーム間の違いを詳細に説明します。

## Architecture Comparison

### iOS Implementation

**Technology Stack:**
- **XCUITest Runner**: UI操作（HID、アクセシビリティ、スクリーンショット、クリップボード）
- **xcrun simctl**: ライフサイクル管理（boot、shutdown、install、uninstall、list_simulators）

**アーキテクチャ:**
```
agent-mobile
    ↓
XCUITestClient (HTTP) ──→ XCUITest Runner (localhost:8200) ──→ iOS Simulator
    ↓
xcrun simctl ───────────────────────────────────→ iOS Simulator
```

**特徴:**
- HTTP/JSON ベースの通信
- XCUITest API による高精度なUI操作
- スクリーンショット、アクセシビリティ、クリップボード対応
- デバイス検出はキャッシュ付き（TTL 5秒）

### Android Implementation

**Technology Stack:**
- **adb (Android Debug Bridge)**: すべての操作
- **UIAutomator**: UI 階層取得
- **input コマンド**: タップ、テキスト入力

**アーキテクチャ:**
```
agent-mobile
    ↓
adb client ──→ adb server (port 5037) ──→ Android Emulator/Device
    ↓                                            ↓
UIAutomator dump ───────────────────→ UI Hierarchy XML
```

**特徴:**
- シンプルなコマンドライン操作
- 追加プロセス不要
- USB デバッグのみで動作
- エミュレータと実機で同じ API

## Feature Availability

| Feature | iOS | Android | Notes |
|---------|-----|---------|-------|
| **Navigation & Interaction** |
| Tap | ✅ | ✅ | 両方とも座標ベース |
| Long Press | ✅ | ✅ | iOS: HID events, Android: input swipe |
| Fill | ✅ | ✅ | 両方ともクリア + type |
| Type | ✅ | ✅ | |
| Swipe | ✅ | ✅ | |
| Scroll | ✅ | ✅ | |
| Special Keys | ✅ (home, siri, etc.) | ⚠️ (limited) | iOS: HID keycode, Android: input keyevent |
| **Element Discovery** |
| Snapshot | ✅ | ✅ | iOS: accessibility_info, Android: uiautomator dump |
| Find | ✅ | ✅ | セマンティックロケーター |
| Get | ✅ | ✅ | 要素プロパティ取得 |
| Is | ✅ | ✅ | 状態チェック |
| Wait | ✅ | ✅ | |
| **Media & Debugging** |
| Screenshot | ✅ | ✅ | iOS: PNG, Android: PNG |
| Record | ✅ | ✅ | iOS: MP4, Android: MP4 |
| Console | ✅ | ✅ | iOS: log stream, Android: logcat |
| **Application Management** |
| Launch | ✅ | ✅ | |
| Terminate | ✅ | ✅ | |
| Install | ✅ (.ipa, .app) | ✅ (.apk) | |
| Uninstall | ✅ | ✅ | |
| List Apps | ✅ | ✅ | |
| Permissions | ✅ | ✅ | 異なる権限名 |
| **Device Management** |
| List | ✅ | ✅ | |
| Boot | ✅ (simctl) | ✅ (emulator) | |
| Shutdown | ✅ (simctl) | ✅ (adb emu kill) | |
| Clipboard Copy | ✅ (pbcopy) | ❌ | Android: adb 経由で非対応 |
| Clipboard Paste | ✅ (pbpaste) | ❌ | Android: adb 経由で非対応 |
| **Advanced** |
| XCTest | ✅ | ❌ | iOS のみ |
| Debugger Attach | ⚠️ (limited) | ⚠️ (limited) | Platform tools required |
| File Transfer | ✅ | ✅ | iOS: simctl, Android: adb push/pull |
| UI Inspector | ✅ (XCUITest) | ⚠️ (uiautomatorviewer) | |

**凡例:**
- ✅ = 完全サポート
- ⚠️ = 部分的サポート/制限あり
- ❌ = 非対応

## Command Differences

### Platform Detection

agent-mobile は自動的にプラットフォームを検出しますが、明示的に指定することもできます。

```bash
# 自動検出
agent-mobile snapshot

# 明示的指定
agent-mobile --platform ios snapshot
agent-mobile --platform android snapshot
```

**検出ロジック:**
1. `simctl list_simulators()` で起動中のシミュレータあり → iOS
2. `adb devices` でデバイスあり → Android
3. デフォルト → iOS

### Tap Command

**iOS:**
```bash
# 座標
agent-mobile tap 100,200

# 特殊キー
agent-mobile tap home
agent-mobile tap siri
agent-mobile tap volumeUp

# 要素参照
agent-mobile tap @e1
```

**Android:**
```bash
# 座標
agent-mobile tap 100,200

# 特殊キー（制限あり）
agent-mobile tap back  # KEYCODE_BACK
agent-mobile tap home  # KEYCODE_HOME

# 要素参照
agent-mobile tap @e1
```

### Snapshot Output

**iOS:**
```
@e1 Button "Login" (enabled)
  frame: (100.0, 200.0, 80.0, 44.0)
  traits: [Button]
@e2 TextField "Email" (enabled)
  placeholder: "Enter your email"
  frame: (50.0, 100.0, 200.0, 40.0)
```

**Android:**
```
@e1 Button "Login" (enabled)
  frame: (100.0, 200.0, 80.0, 44.0)
  content-desc: "Login button"
@e2 EditText "Email" (enabled)
  hint: "Enter your email"
  frame: (50.0, 100.0, 200.0, 40.0)
```

### Permissions

**iOS Permissions:**
```bash
agent-mobile app grant camera -b com.example.app
agent-mobile app grant location -b com.example.app
agent-mobile app grant photos -b com.example.app
agent-mobile app grant contacts -b com.example.app
agent-mobile app grant microphone -b com.example.app
```

**iOS 権限リスト:**
`camera`, `location`, `contacts`, `photos`, `microphone`, `calendar`, `reminders`, `siri`, `speech-recognition`, `health`, `homekit`, `faceid`, `motion`, `media-library`

**Android Permissions:**
```bash
agent-mobile app grant camera -b com.example.app
agent-mobile app grant location -b com.example.app
agent-mobile app grant contacts -b com.example.app
agent-mobile app grant storage -b com.example.app
agent-mobile app grant microphone -b com.example.app
```

**Android 権限リスト:**
`camera`, `location`, `location-coarse`, `contacts`, `calendar`, `storage`, `microphone`, `phone`, `sms`

## Element Type Mapping

### iOS Element Types

```
Button              → タップ可能なボタン
TextField           → テキスト入力フィールド
SecureTextField     → パスワード入力フィールド
StaticText          → 静的テキストラベル
Image               → 画像
Cell                → テーブル/コレクションセル
Switch              → トグルスイッチ
Slider              → スライダー
TabBar              → タブバー
NavigationBar       → ナビゲーションバー
TextView            → 複数行テキスト
ScrollView          → スクロールビュー
Table               → テーブルビュー
```

### Android Element Types

```
Button              → タップ可能なボタン
EditText            → テキスト入力フィールド
TextView            → テキストビュー
ImageView           → 画像ビュー
CheckBox            → チェックボックス
RadioButton         → ラジオボタン
Switch              → スイッチ
SeekBar             → シークバー
RecyclerView        → リサイクラービュー
ListView            → リストビュー
ScrollView          → スクロールビュー
WebView             → ウェブビュー
```

### Common Mappings

| Concept | iOS | Android |
|---------|-----|---------|
| テキスト入力 | TextField | EditText |
| パスワード入力 | SecureTextField | EditText (inputType=textPassword) |
| ボタン | Button | Button |
| ラベル | StaticText | TextView |
| 画像 | Image | ImageView |
| リストアイテム | Cell | RecyclerView.ViewHolder |
| トグル | Switch | Switch |

## Coordinate Systems

### iOS

- **原点**: 左上 (0, 0)
- **単位**: Points（論理ピクセル）
- **座標系**: UIKit 座標系

```bash
# iPhone 15 (390x844 points)
agent-mobile tap 195,422  # 画面中央
```

**注意:**
Retina ディスプレイでは、物理ピクセルは points の 2-3 倍です。

### Android

- **原点**: 左上 (0, 0)
- **単位**: Pixels（物理ピクセル）
- **座標系**: Android View 座標系

```bash
# Pixel 7 (1080x2400 pixels)
agent-mobile tap 540,1200  # 画面中央
```

**注意:**
デバイスの DPI により、同じ座標でも異なる物理位置になります。

## Text Input Methods

### iOS

**HID Events を使用:**
```rust
// 内部実装（参考）
let text_events = events::text_to_events(&text)?;
client.hid(text_events).await?;
```

**特徴:**
- キーボード入力を完全にシミュレート
- 特殊文字対応
- キーボードレイアウト非依存

**制限:**
- 絵文字は制限あり
- 一部の非ASCII文字は非対応

### Android

**adb input text を使用:**
```bash
adb shell input text "Hello%sWorld"  # スペースは %s
```

**特徴:**
- シンプル
- 日本語対応（要エンコーディング）
- 高速

**制限:**
- スペースは `%s` にエスケープ必要
- 特殊文字の制限

## Performance Characteristics

### iOS

**スナップショット取得:**
- **速度**: 約 500ms
- **方法**: XCUITest Runner accessibility API
- **制限**: 大きな UI では遅くなる（WebView含むアプリは60秒以上の場合あり）

**タップ操作:**
- **速度**: 約 100ms
- **方法**: XCUITest Runner HTTP API

**アプリ起動:**
- **速度**: 約 2-3秒
- **方法**: simctl launch

### Android

**スナップショット取得:**
- **速度**: 約 300ms
- **方法**: uiautomator dump
- **制限**: XML パース時間

**タップ操作:**
- **速度**: 約 50ms
- **方法**: adb shell input tap

**アプリ起動:**
- **速度**: 約 1-2秒
- **方法**: am start

**パフォーマンス比較:**
| 操作 | iOS | Android | 差 |
|------|-----|---------|-----|
| Snapshot | 500ms | 300ms | Android が 1.7x 速い |
| Tap | 100ms | 50ms | Android が 2x 速い |
| Launch | 2-3s | 1-2s | Android がやや速い |

## Known Issues

### iOS

**1. Permission Approval Schema Error**
- **問題**: 一部の権限で SQLite スキーマエラー
- **回避策**: simctl にフォールバック（自動）

**2. Simulator Boot Timeout**
- **問題**: シミュレータ起動に時間がかかる
- **回避策**: headless モードを使用

### Android

**1. Clipboard Not Supported**
- **問題**: adb 経由でクリップボード操作非対応
- **回避策**: なし（プラットフォーム制限）

**2. Emulator Startup Slow**
- **問題**: エミュレータ起動が遅い
- **回避策**: スナップショットを有効化

**3. UIAutomator Dump Timeout**
- **問題**: 複雑な UI でタイムアウト
- **回避策**: --no-scroll を使用

## Best Practices

### Cross-Platform Testing

**同じテストを両方で実行:**
```bash
#!/bin/bash

run_test() {
  local platform=$1
  local session=$2

  agent-mobile --session "$session" app launch com.example.app
  sleep 2
  agent-mobile --session "$session" snapshot -i
  agent-mobile --session "$session" find label "Email" fill "test@example.com"
  agent-mobile --session "$session" find label "Password" fill "password123"
  agent-mobile --session "$session" find text "Login" tap
  sleep 2
  agent-mobile --session "$session" screenshot -o "${platform}_result.png"
}

# iOS
agent-mobile session create ios --udid "ABC-123" -p ios
run_test ios ios

# Android
agent-mobile session create android --udid "emulator-5554" -p android
run_test android android
```

### Platform-Specific Code

必要に応じてプラットフォーム固有のコードを分岐:

```bash
#!/bin/bash

PLATFORM=$(agent-mobile device list -f json | jq -r '.[0].platform')

if [ "$PLATFORM" = "ios" ]; then
  # iOS 固有の処理
  agent-mobile tap home
  agent-mobile device pbcopy "Hello"
elif [ "$PLATFORM" = "android" ]; then
  # Android 固有の処理
  agent-mobile tap back
fi
```

### Permission Mapping

プラットフォーム間で権限名をマッピング:

```bash
#!/bin/bash

grant_permission() {
  local perm=$1
  local bundle=$2
  local platform=$3

  case "$platform" in
    ios)
      agent-mobile app grant "$perm" -b "$bundle"
      ;;
    android)
      # Android の権限名にマッピング
      case "$perm" in
        photos) perm="storage" ;;
        location-always) perm="location" ;;
      esac
      agent-mobile app grant "$perm" -b "$bundle"
      ;;
  esac
}

grant_permission "camera" "com.example.app" "ios"
grant_permission "camera" "com.example.app" "android"
```

## Related Concepts

- [SKILL.md - Platform-Specific Notes](../SKILL.md#platform-specific-notes) - プラットフォーム固有の注意事項

---

**Last Updated**: 2026-01-23
