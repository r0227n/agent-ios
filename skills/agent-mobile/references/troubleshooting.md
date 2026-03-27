# Troubleshooting Guide

agent-mobile の一般的な問題と解決策の完全ガイド。

## Quick Diagnosis

問題が発生したら、まず以下を確認してください:

```bash
# 1. デバイスが接続されているか
agent-mobile device list

# 2. アプリが起動しているか
agent-mobile app list

# 3. UI の現在の状態
agent-mobile snapshot -i

# 4. スクリーンショットで視覚的に確認
agent-mobile screenshot -o debug.png
```

## Common Errors

### 1. "No Image available to encode"

**エラーメッセージ:**
```
Error: No Image available to encode
```

**原因:**
スクリーンショットを取得しようとしたが、画面に表示されるものがない。

**一般的なシナリオ:**
- アプリが起動していない
- アプリがバックグラウンドにある
- シミュレータ/エミュレータがシャットダウンしている

**解決策:**

1. アプリを起動する:
```bash
agent-mobile app launch com.example.app
sleep 2
agent-mobile screenshot
```

2. デバイスが起動しているか確認:
```bash
agent-mobile device list
# State が "Booted" または "device" であることを確認
```

3. アプリをフォアグラウンドに:
```bash
agent-mobile app launch com.example.app --foreground-if-running
```

### 2. "No elements found matching..."

**エラーメッセージ:**
```
Error: No elements found matching text containing "Login"

Hint: Run 'agent-mobile snapshot' to see the current UI state.
```

**原因:**
セマンティックロケーターが UI 上のどの要素にもマッチしなかった。

**一般的なシナリオ:**
- 要素が存在しない（画面遷移していない）
- タイポ（"Login" vs "Log in"）
- 大文字小文字の問題（通常は自動で無視されるが）
- 要素が画面外（スクロールが必要）

**解決策:**

1. UI の状態を確認:
```bash
agent-mobile snapshot -i > debug.txt
cat debug.txt
```

2. 部分一致を使用（--exact を外す）:
```bash
# NG: 完全一致
agent-mobile find text "Login" --exact

# OK: 部分一致
agent-mobile find text "Login"  # "Login Now" にもマッチ
```

3. 別のロケーターを試す:
```bash
# text の代わりに type
agent-mobile find type Button --first

# label を試す
agent-mobile find label "Login"
```

4. スクロールして要素を表示:
```bash
agent-mobile scroll down
sleep 1
agent-mobile find text "Login" tap
```

### 3. "Element ref @eN not found"

**エラーメッセージ:**
```
Error: Element ref @e1 not found in current snapshot
```

**原因:**
スナップショット取得後に画面が変更され、参照が無効になった。

**一般的なシナリオ:**
- ページ遷移後に古い参照を使用
- スナップショットを取得せずに参照を使用
- 別セッションの参照を使用

**解決策:**

1. 新しいスナップショットを取得:
```bash
agent-mobile snapshot
agent-mobile tap @e1  # 新しい参照
```

2. 画面遷移後は必ず再取得:
```bash
agent-mobile snapshot
agent-mobile tap @e1  # ページ遷移
sleep 1
agent-mobile snapshot  # 新しいスナップショット
agent-mobile tap @e1   # 新しい画面の @e1
```

3. 参照の存在を確認:
```bash
agent-mobile snapshot | grep "@e10"
# 存在すれば表示される
```

### 4. "Session 'xxx' not found"

**エラーメッセージ:**
```
Error: Session 'xxx' not found
```

**原因:**
指定されたセッションが作成されていない。

**解決策:**

1. セッション一覧を確認:
```bash
agent-mobile session list
```

2. セッションを作成:
```bash
agent-mobile session create xxx --udid ABC-123 -p ios
```

3. 環境変数を確認:
```bash
echo $AGENT_MOBILE_SESSION
# 意図しないセッション名が設定されていないか確認
```

### 5. "No device found"

**エラーメッセージ:**
```
Error: No device found. Please connect an iOS simulator/device or Android emulator/device.
```

**原因:**
接続されているデバイスがない。

**解決策（iOS）:**

1. シミュレータを起動:
```bash
agent-mobile device boot "iPhone 15"
sleep 5
agent-mobile device list
```

2. XCUITest Runner が起動しているか確認:
```bash
# XCUITest Runner は agent-mobile コマンド実行時に自動起動されます
agent-mobile device list
```

**解決策（Android）:**

1. エミュレータを起動:
```bash
agent-mobile device boot Pixel_7
sleep 10
agent-mobile device list
```

2. adb が動作しているか確認:
```bash
adb devices
```

3. adb サーバーを再起動:
```bash
adb kill-server
adb start-server
adb devices
```

### 6. "Permission denied" or "Access denied"

**エラーメッセージ:**
```
Error: Permission denied
```

**原因:**
- iOS: XCUITest Runner のアクセス権限がない
- Android: USB デバッグが無効

**解決策（iOS）:**

1. XCUITest Runner に権限を付与:
```bash
# System Preferences → Security & Privacy → Privacy
# Accessibility と Automation で許可
```

2. デバイスを再起動:
```bash
agent-mobile device shutdown <udid>
agent-mobile device boot <name>
```

**解決策（Android）:**

1. USB デバッグを有効化:
```
Settings → Developer Options → USB Debugging
```

2. RSA キーを承認（実機の場合）:
```
デバイスに表示されるダイアログで "Always allow" を選択
```

### 7. Timeout Errors

**エラーメッセージ:**
```
Error: Operation timed out
```

**原因:**
- デバイスの応答が遅い
- 複雑な UI でスナップショット取得に時間がかかる
- ネットワークの問題（実機の場合）

**解決策:**

1. タイムアウト時間を延長:
```bash
agent-mobile wait @e1 --timeout 30  # 30秒に延長
```

2. --no-scroll を使用:
```bash
agent-mobile snapshot --no-scroll  # スクロール無効化
```

3. デバイスを再起動:
```bash
agent-mobile device shutdown <udid>
agent-mobile device boot <name>
```

### 8. "Failed to launch app"

**エラーメッセージ:**
```
Error: Failed to launch app: com.example.app
```

**原因:**
- アプリがインストールされていない
- Bundle ID が間違っている
- アプリがクラッシュしている

**解決策:**

1. アプリがインストールされているか確認:
```bash
agent-mobile app list | grep com.example.app
```

2. アプリをインストール:
```bash
# iOS
agent-mobile app install /path/to/app.ipa

# Android
agent-mobile app install /path/to/app.apk
```

3. Bundle ID を確認:
```bash
# iOS
agent-mobile app list -f json | jq '.[] | select(.name | contains("Example"))'

# Android
agent-mobile app list -f json | jq '.[] | select(.bundle_id | contains("example"))'
```

4. コンソールログでクラッシュ情報を確認:
```bash
agent-mobile console --level error
```

## Platform-Specific Issues

### iOS

#### "Simulator stuck at 'Booting'"

**症状:**
シミュレータが起動中のままフリーズ。

**解決:**
```bash
# シミュレータをシャットダウン
agent-mobile device shutdown <udid>

# または xcrun simctl で強制終了
xcrun simctl shutdown all

# 再起動
agent-mobile device boot "iPhone 15"
```

#### "Permission Approval Failed"

**症状:**
権限付与コマンドが失敗する。

**解決:**
```bash
# simctl フォールバックを使用（自動）
agent-mobile app grant camera -b com.example.app

# または手動で simctl を使用
xcrun simctl privacy <udid> grant camera com.example.app
```

### Android

#### "Device offline"

**症状:**
`adb devices` で "offline" と表示される。

**解決:**
```bash
# USB ケーブルを抜き差し（実機の場合）

# または adb を再起動
adb kill-server
adb start-server
adb devices
```

#### "Installation failed: INSTALL_FAILED_INSUFFICIENT_STORAGE"

**症状:**
アプリインストール時にストレージ不足エラー。

**解決:**
```bash
# エミュレータのストレージを増やす
# AVD Manager → Edit → Advanced Settings → Internal Storage

# または古いアプリをアンインストール
agent-mobile app uninstall com.old.app
```

#### "UIAutomator dump failed"

**症状:**
スナップショット取得時にエラー。

**解決:**
```bash
# --no-scroll を使用
agent-mobile snapshot --no-scroll

# または adb を再起動
adb kill-server
adb start-server
```

## Performance Issues

### Slow Snapshot

**症状:**
スナップショット取得に5秒以上かかる。

**解決:**

1. インタラクティブ要素のみ取得:
```bash
agent-mobile snapshot -i
```

2. 深さを制限:
```bash
agent-mobile snapshot -d 3
```

3. スクロール無効化:
```bash
agent-mobile snapshot --no-scroll
```

4. コンパクト表示:
```bash
agent-mobile snapshot -i -c -d 3
```

### Slow App Launch

**症状:**
アプリ起動に10秒以上かかる。

**解決:**

1. キャッシュをクリア（iOS）:
```bash
xcrun simctl erase <udid>
```

2. エミュレータを再起動（Android）:
```bash
agent-mobile device shutdown <serial>
agent-mobile device boot <name>
```

3. ハードウェアアクセラレーションを有効化（Android）:
```
AVD Manager → Edit → Emulated Performance → Graphics: Hardware
```

## Debugging Techniques

### 1. Verbose Output

環境変数でデバッグ情報を有効化:

```bash
export RUST_LOG=debug
agent-mobile snapshot
```

### 2. Save Snapshots

スナップショットをファイルに保存して分析:

```bash
agent-mobile snapshot -o debug.txt
agent-mobile snapshot -o debug.json -f json

# JSON で詳細に分析
cat debug.json | jq '.elements[] | select(.enabled == false)'
```

### 3. Console Monitoring

コンソールログを監視してエラーを検出:

```bash
agent-mobile console --level error > errors.log &
# テスト実行...
# errors.log を確認
```

### 4. Screenshot Comparison

スクリーンショットで視覚的に確認:

```bash
agent-mobile screenshot -o before.png
agent-mobile tap @e1
sleep 1
agent-mobile screenshot -o after.png

# 画像を並べて比較
open before.png after.png
```

### 5. Element Inspection

要素の詳細情報を JSON で取得:

```bash
agent-mobile find type Button -f json | jq
agent-mobile get @e1 label
agent-mobile get @e1 value
agent-mobile is @e1 enabled
```

## Recovery Procedures

### Complete Reset

すべてをリセットしてクリーンスタート:

**iOS:**
```bash
# 1. シミュレータをシャットダウン
xcrun simctl shutdown all

# 3. シミュレータをリセット（オプション）
xcrun simctl erase all

# 4. シミュレータを起動
agent-mobile device boot "iPhone 15"

# 5. アプリをインストール
agent-mobile app install /path/to/app.ipa
```

**Android:**
```bash
# 1. adb サーバーを停止
adb kill-server

# 2. エミュレータをシャットダウン
agent-mobile device shutdown <serial>

# 3. エミュレータをワイプ（オプション）
emulator -avd <name> -wipe-data

# 4. エミュレータを起動
agent-mobile device boot <name>

# 5. アプリをインストール
agent-mobile app install /path/to/app.apk
```

### Session Reset

セッションをリセット:

```bash
# セッションを削除
agent-mobile session rm my-session

# セッションを再作成
agent-mobile session create my-session --udid <udid> -p <platform>
```

## Getting Help

問題が解決しない場合:

1. **Issue を報告:**
   - GitHub: https://github.com/anthropics/claude-code/issues
   - 詳細な情報を含める（エラーメッセージ、環境、再現手順）

2. **デバッグ情報を収集:**
```bash
# システム情報
uname -a
agent-mobile --version

# デバイス情報
agent-mobile device list -f json

# エラーログ
export RUST_LOG=debug
agent-mobile <command> 2>&1 | tee debug.log
```

3. **最小再現例を作成:**
```bash
#!/bin/bash
# minimal-repro.sh

set -e
agent-mobile device list
agent-mobile app launch com.example.app
agent-mobile snapshot
# ... 問題が発生するコマンド
```

## Related Resources

- [SKILL.md - Error Handling](../SKILL.md#error-handling) - エラーハンドリングの概要
- [Platform Differences](platform-differences.md) - プラットフォーム固有の問題
- [Element References](element-references.md) - 参照関連のエラー
- [Session Management](session-management.md) - セッション関連のエラー

---

**Last Updated**: 2026-01-23
