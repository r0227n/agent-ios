---
name: agent-mobile
description: Mobile app E2E testing CLI for AI agents with semantic locators and element references
user-invocable: true
allowed-tools: [Bash, Read, Write, Glob, Grep]
---

# agent-mobile

AI-powered mobile app E2E testing CLI with semantic locators and efficient element reference system.

## Overview

`agent-mobile` は AI エージェント向けのモバイルアプリ E2E テストツールです。iOS シミュレータ、Android エミュレータ、実機をサポートし、自然言語に近いセマンティックロケーターと効率的な要素参照システム (`@e1`, `@e2`) を提供します。

**主な特徴:**
- **Semantic Locators**: テキスト、タイプ、ラベルで UI 要素を検索
- **Element References**: トークン効率的な `@e1` 形式の短縮参照
- **Cross-Platform**: iOS と Android を統一 API でサポート
- **Session Management**: 永続的な状態管理でマルチデバイスワークフロー対応
- **JSON Output**: すべてのコマンドで `--json` オプションをサポート
- **Auto-Detection**: プラットフォームとデバイスを自動検出

## Quick Start

```bash
# 1. UI スナップショットを取得（要素参照を生成）
agent-mobile snapshot

# 2. セマンティックロケーターで要素を検索してアクション実行
agent-mobile find text "Login" tap
agent-mobile find label "Email" fill "test@example.com"

# 3. 要素参照を使用（トークン効率的）
agent-mobile tap @e1
agent-mobile fill @e2 "password"

# 4. スクリーンショットで検証
agent-mobile screenshot
```

## Core Concepts

### Element References (`@e1`, `@e2`)

**最も重要な概念**: トークン使用量を劇的に削減する短縮 ID システム。

```bash
# snapshot を取得すると @e1, @e2, ... が生成される
agent-mobile snapshot
# 出力:
# @e1 Button "Login" (enabled)
# @e2 TextField "Email" (enabled)
# @e3 SecureTextField "Password" (enabled)

# 参照を使用（繰り返しスナップショット不要）
agent-mobile tap @e1
agent-mobile fill @e2 "user@example.com"
agent-mobile fill @e3 "password123"
```

**重要な制約:**
- 参照は最後の `snapshot` に紐づく（画面変更後は無効）
- 画面外の要素は参照できない
- セッションをまたいで永続化しない

詳細: [references/element-references.md](references/element-references.md)

### Semantic Locators

UI 要素を人間が理解しやすい方法で検索:

```bash
# タイプで検索
agent-mobile find type Button

# テキスト内容で検索（label または value）
agent-mobile find text "Login"

# ラベルで検索
agent-mobile find label "Email"

# プレースホルダーで検索
agent-mobile find placeholder "Enter your email"

# 状態で検索
agent-mobile find enabled
agent-mobile find disabled
```

**位置指定:**
- `--first` (デフォルト): 最初の要素
- `--last`: 最後の要素
- `--nth N`: N 番目の要素（0-indexed）
- `--all`: すべての要素（アクションなし時のみ）

**マッチングモード:**
- デフォルト: 部分一致（大文字小文字区別なし）
- `--exact`: 完全一致

詳細: [references/semantic-locators.md](references/semantic-locators.md)

### Session Management

複数デバイスで並行作業する際の状態管理:

```bash
# セッション作成
agent-mobile session create ios-dev --udid ABC-123 -p ios
agent-mobile session create android-dev --udid emulator-5554 -p android

# セッションを使用
agent-mobile --session ios-dev snapshot
agent-mobile --session android-dev snapshot

# または環境変数で設定
export AGENT_MOBILE_SESSION=ios-dev
agent-mobile snapshot
```

詳細: [references/session-management.md](references/session-management.md)

## Command Reference

### Navigation & Interaction

| Command | Description | Example |
|---------|-------------|---------|
| `tap <target>` | 要素または座標をタップ | `tap @e1`, `tap 100,200`, `tap home` |
| `long-press <target>` | 長押し（デフォルト1秒） | `long-press @e1`, `long-press @e1 --duration 2.0` |
| `fill <target> <text>` | テキストフィールドをクリア + 入力 | `fill @e2 "test@example.com"` |
| `type <text>` | フォーカス中のフィールドに追記 | `type "additional text"` |
| `swipe <direction>` | スワイプジェスチャー | `swipe up`, `swipe down`, `swipe left`, `swipe right` |
| `scroll <direction>` | 要素内または画面をスクロール | `scroll down`, `scroll up --element @e1` |

**Tap Targets:**
- Element reference: `@e1`, `@e2`
- Coordinates: `100,200` (x,y)
- Special keys (iOS): `home`, `lock`, `siri`, `volumeUp`, `volumeDown`, `applePay`

### Element Discovery

| Command | Description | Example |
|---------|-------------|---------|
| `snapshot` | UI 階層を要素参照付きで取得 | `snapshot`, `snapshot -i` (interactive only), `snapshot -c` (compact) |
| `find <locator>` | セマンティックロケーターで検索 | `find type Button`, `find text "Login"` |
| `find <locator> <action>` | 検索 + アクション実行 | `find text "Login" tap`, `find label "Email" fill "user@example.com"` |
| `get <target> <property>` | 要素プロパティを取得 | `get @e1 text`, `get @e1 label`, `get @e1 value` |
| `is <target> <condition>` | 状態チェック（exit code） | `is @e1 enabled`, `is @e2 disabled` |
| `wait <target>` | 要素出現を待機 | `wait @e1`, `wait @e1 --timeout 10` |

**Snapshot Options:**
- `-i, --interactive`: インタラクティブ要素のみ表示
- `-c, --compact`: 空要素を削除
- `-d, --depth N`: 階層深さを N に制限
- `-o, --output <file>`: ファイルに保存
- `-f, --format json`: JSON 形式で出力
- `--no-scroll`: スクロール無効化（デフォルトは有効）

**Find Locators:**
- `type <element-type>`: 要素タイプで検索 (Button, TextField, etc.)
- `text <text>`: テキスト内容で検索 (label または value)
- `label <label>`: ラベルで検索
- `placeholder <placeholder>`: プレースホルダーで検索
- `enabled`: 有効な要素
- `disabled`: 無効な要素

**Find Actions:**
- `tap`: タップ
- `long-press`: 長押し
- `fill <text>`: クリア + 入力
- `clear`: クリア

### Media & Debugging

| Command | Description | Example |
|---------|-------------|---------|
| `screenshot` | スクリーンショットを PNG で保存 | `screenshot`, `screenshot -o result.png` |
| `record start` | 画面録画を開始 | `record start`, `record start -o video.mp4` |
| `record stop` | 画面録画を停止 | `record stop` |
| `console` | デバイスログをストリーミング | `console`, `console --level error` |

### Application Management

| Command | Description | Example |
|---------|-------------|---------|
| `app launch <bundle-id>` | アプリを起動 | `app launch com.example.app` |
| `app terminate <bundle-id>` | アプリを終了 | `app terminate com.example.app` |
| `app install <path>` | アプリをインストール | `app install app.ipa`, `app install app.apk` |
| `app uninstall <bundle-id>` | アプリをアンインストール | `app uninstall com.example.app` |
| `app list` | インストール済みアプリ一覧 | `app list`, `app list -f json` |
| `app grant <perm> -b <bundle>` | 権限を付与 | `app grant camera -b com.example.app` |
| `app revoke <perm> -b <bundle>` | 権限を取り消し | `app revoke camera -b com.example.app` |
| `app reset <perm> -b <bundle>` | 権限をリセット | `app reset camera -b com.example.app` |

**iOS Permissions:**
`camera`, `location`, `contacts`, `photos`, `microphone`, `calendar`, `reminders`, `siri`, `health`, `homekit`, etc.

**Android Permissions:**
`camera`, `location`, `contacts`, `calendar`, `storage`, `microphone`, `phone`, `sms`, etc.

### Device Management

| Command | Description | Example |
|---------|-------------|---------|
| `device list` | デバイス/シミュレータ一覧 | `device list`, `device list -p ios` |
| `device boot <name>` | シミュレータ/エミュレータを起動 | `device boot "iPhone 15"`, `device boot Pixel_7` |
| `device shutdown <udid>` | デバイスをシャットダウン | `device shutdown ABC-123` |
| `device pbcopy <text>` | クリップボードにコピー（iOS のみ） | `device pbcopy "Hello"` |
| `device pbpaste` | クリップボードから取得（iOS のみ） | `device pbpaste` |

### Session Management

| Command | Description | Example |
|---------|-------------|---------|
| `session create <name>` | セッションを作成 | `session create test --udid ABC-123 -p ios` |
| `session list` | セッション一覧 | `session list`, `session list -f json` |
| `session show` | 現在のセッション情報 | `session show` (requires --session) |
| `session destroy <name>` | セッションを削除 | `session destroy test` |

### IDB Commands (80+ iOS-specific)

`agent-mobile idb <command>` でフル IDB CLI 互換コマンドを実行可能:

**よく使われるコマンド:**
- `idb list-targets`: 接続デバイス一覧
- `idb launch <bundle-id>`: アプリ起動（詳細オプション付き）
- `idb log`: デバイスログ取得
- `idb focus`: シミュレータウィンドウを前面に
- `idb ui-server`: Xcode Inspector サーバー起動
- `idb file push/pull`: ファイル転送
- `idb xctest`: XCTest 実行
- `idb debugserver`: デバッガアタッチ

完全なリスト: `agent-mobile idb --help`

## Global Options

すべてのコマンドで使用可能:

| Option | Description | Example |
|--------|-------------|---------|
| `--session <name>` | セッション名（AGENT_MOBILE_SESSION 環境変数も可） | `--session ios-dev` |
| `--udid <udid>` | デバイス UDID（指定しない場合は自動検出） | `--udid ABC-123` |
| `--platform <platform>` | プラットフォーム（ios または android、通常は自動検出） | `--platform ios` |
| `-f, --format json` | JSON 形式で出力 | `-f json` |

## Common Workflows

### 1. Basic Login Flow

```bash
# 1. アプリを起動
agent-mobile app launch com.example.app

# 2. UI をスナップショット
agent-mobile snapshot -i  # インタラクティブ要素のみ

# 3. セマンティックロケーターでログイン
agent-mobile find label "Email" fill "user@example.com"
agent-mobile find label "Password" fill "password123"
agent-mobile find text "Login" tap

# 4. 成功を確認
agent-mobile wait @e1 --timeout 5
agent-mobile screenshot -o login_success.png
```

### 2. Form Automation with Element References

```bash
# 1. スナップショット取得
agent-mobile snapshot > snapshot.txt

# 2. 参照を使用して高速入力
agent-mobile fill @e1 "John"
agent-mobile fill @e2 "Doe"
agent-mobile fill @e3 "john@example.com"
agent-mobile tap @e4  # Submit button
```

### 3. Multi-Device Testing

```bash
# セッション作成
agent-mobile session create ios16 --udid ABC-123 -p ios
agent-mobile session create android13 --udid emulator-5554 -p android

# 並行実行
agent-mobile --session ios16 app launch com.example.app &
agent-mobile --session android13 app launch com.example.app &

# 同じテストを両方で実行
for session in ios16 android13; do
  agent-mobile --session $session find text "Login" tap
  agent-mobile --session $session screenshot -o ${session}_result.png
done
```

### 4. Regression Testing

```bash
# ベースラインスナップショット
agent-mobile snapshot -o baseline.json -f json

# ... コード変更後 ...

# 新しいスナップショット
agent-mobile snapshot -o current.json -f json

# 差分比較（外部ツール使用）
diff baseline.json current.json
```

## Future Commands (Planned)

以下のコマンドは現在実装中です（`todo.md` を参照）:

### Priority: High (Phase 1)
- `is visible <target>` - 可視性判定
- `is checked <target>` - チェック状態判定
- `get count <locator>` - マッチング要素数
- `get box <target>` - バウンディングボックス取得
- `wait --text <text>` - テキスト出現待機
- `wait --timeout <ms>` - タイムアウト設定

### Priority: Medium (Phase 2)
- `check <target>` - チェックボックス/スイッチをオン
- `uncheck <target>` - チェックボックス/スイッチをオフ
- `get attr <target> <attribute>` - 任意属性取得
- `select <target> <value>` - Picker/ドロップダウン選択

### Priority: Low (Phase 3)
- `snapshot -s <selector>` - スコープ限定スナップショット

これらのコマンドの詳細な仕様は `/Users/r0227n/Dev/agent-mobile/todo.md` を参照してください。

## Best Practices

### Token Efficiency

**Always use element references after snapshot:**

❌ Bad (繰り返しスナップショット):
```bash
agent-mobile snapshot
agent-mobile find text "Email" fill "user@example.com"
agent-mobile snapshot  # 不要！
agent-mobile find text "Password" fill "password"
```

✅ Good (参照を再利用):
```bash
agent-mobile snapshot
# @e1: Email TextField
# @e2: Password SecureTextField
agent-mobile fill @e1 "user@example.com"
agent-mobile fill @e2 "password"
```

**スナップショットのフィルタリング:**
```bash
# 大きなアプリでは -i (interactive only) を使用
agent-mobile snapshot -i

# さらに -c (compact) と -d (depth) で絞り込み
agent-mobile snapshot -i -c -d 3
```

### Reliability

**明示的な待機を使用:**
```bash
# アニメーション後に待機
agent-mobile tap @e1
sleep 1  # または wait コマンド
agent-mobile screenshot
```

**エラーハンドリング:**
```bash
# is コマンドの exit code を使用
if agent-mobile is @e1 enabled; then
  agent-mobile tap @e1
else
  echo "Button is disabled"
fi
```

### Debugging

**スナップショットを保存して分析:**
```bash
agent-mobile snapshot -o debug.json -f json
agent-mobile snapshot -o debug.txt
```

**スクリーンショットで証跡を残す:**
```bash
agent-mobile screenshot -o before.png
agent-mobile tap @e1
agent-mobile screenshot -o after.png
```

**コンソールログを監視:**
```bash
agent-mobile console --level error &
# テスト実行...
```

### AI Agent Optimization

**JSON 出力を活用:**
```bash
# プログラムで解析可能な出力
devices=$(agent-mobile device list -f json)
echo "$devices" | jq -r '.[0].udid'
```

**セッションで状態を永続化:**
```bash
# セッションは最後のスナップショットを保存
export AGENT_MOBILE_SESSION=test
agent-mobile snapshot  # 保存される
agent-mobile tap @e1   # セッションから参照を解決
```

**エラーメッセージを読む:**
```bash
agent-mobile find text "NonExistent" tap
# エラー: "No elements found matching text containing "NonExistent"
# Hint: Run 'agent-mobile snapshot' to see the current UI state."
```

## Platform-Specific Notes

### iOS

- **Implementation**: idb gRPC (高機能) + xcrun simctl (ライフサイクル管理)
- **Device Types**: Simulator, Physical Device (要ペアリング)
- **Advantages**: フル IDB CLI 互換、アクセシビリティ API、XCTest サポート
- **Limitations**: idb_companion プロセスが必要、一部機能はシミュレータのみ

### Android

- **Implementation**: adb (Android Debug Bridge)
- **Device Types**: Emulator, Physical Device
- **Advantages**: 追加セットアップ不要、USB デバッグのみ
- **Limitations**: クリップボード API 非対応、一部機能制限あり

詳細: [references/platform-differences.md](references/platform-differences.md)

## Error Handling

### Common Errors

**"No Image available to encode"**
- 原因: スクリーンショット取得時にアプリが起動していない
- 解決: `agent-mobile app launch <bundle-id>` でアプリを起動

**"No elements found matching..."**
- 原因: セマンティックロケーターがマッチしない
- 解決: `agent-mobile snapshot` で UI 状態を確認

**"Element ref @e1 not found"**
- 原因: スナップショット取得後に画面が変更された
- 解決: `agent-mobile snapshot` で再取得

**"Session 'xxx' not found"**
- 原因: セッションが作成されていない
- 解決: `agent-mobile session create xxx --udid ...`

詳細: [references/troubleshooting.md](references/troubleshooting.md)

## Examples

### Complete Login Test

```bash
#!/bin/bash
set -e

# Setup
BUNDLE_ID="com.example.app"
agent-mobile app launch $BUNDLE_ID

# Wait for app to load
sleep 2

# Capture UI
agent-mobile snapshot -i -o login_screen.txt

# Login
agent-mobile find label "Email" fill "test@example.com"
agent-mobile find label "Password" fill "password123"
agent-mobile find text "Login" tap

# Wait for next screen
sleep 2
agent-mobile wait @e1 --timeout 10

# Verify
agent-mobile screenshot -o logged_in.png
echo "Login successful"
```

### Form Submission with Validation

```bash
#!/bin/bash

# Take snapshot
agent-mobile snapshot > /tmp/form.txt

# Extract refs
EMAIL_REF=$(grep "Email" /tmp/form.txt | grep -o "@e[0-9]*" | head -1)
PASSWORD_REF=$(grep "Password" /tmp/form.txt | grep -o "@e[0-9]*" | head -1)
SUBMIT_REF=$(grep "Submit" /tmp/form.txt | grep -o "@e[0-9]*" | head -1)

# Fill form
agent-mobile fill $EMAIL_REF "user@example.com"
agent-mobile fill $PASSWORD_REF "securepass123"

# Check if submit button is enabled
if agent-mobile is $SUBMIT_REF enabled; then
  agent-mobile tap $SUBMIT_REF
  echo "Form submitted"
else
  echo "Submit button is disabled"
  exit 1
fi
```

## References

- [Semantic Locators](references/semantic-locators.md) - 詳細な検索パターンとマッチングルール
- [Element References](references/element-references.md) - @e1 参照システムの完全ガイド
- [Session Management](references/session-management.md) - マルチデバイスワークフローのベストプラクティス
- [Platform Differences](references/platform-differences.md) - iOS vs Android の実装詳細
- [gRPC vs simctl](references/grpc-vs-simctl.md) - アーキテクチャの設計思想
- [Troubleshooting](references/troubleshooting.md) - 一般的な問題と解決策

## Templates

- [Login Workflow](templates/login-workflow.sh) - 基本的なログインフロー
- [Form Automation](templates/form-automation.sh) - 複雑なフォーム自動化
- [UI Exploration](templates/ui-exploration.sh) - UI 探索スクリプト
- [Multi-Device Test](templates/multi-device-test.sh) - iOS/Android 並行テスト
- [Regression Test](templates/regression-test.sh) - リグレッションテスト

## Additional Resources

- **Project Repository**: `/Users/r0227n/Dev/agent-mobile`
- **Architecture Documentation**: `/Users/r0227n/Dev/agent-mobile/docs/ARCHITECTURE.md`
- **Implementation Guide**: `/Users/r0227n/Dev/agent-mobile/CLAUDE.md`
- **Feature Roadmap**: `/Users/r0227n/Dev/agent-mobile/todo.md`
- **Known Issues**: `/Users/r0227n/Dev/agent-mobile/docs/KNOWN_ISSUES.md` (if exists)

---

**Version**: 1.0.0
**Last Updated**: 2026-01-23
**Maintained by**: Claude Code (Anthropic)
