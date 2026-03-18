# Session Management

agent-mobile のセッション管理システムの完全ガイド。

## Overview

セッション管理は、複数のデバイスで並行してテストを実行したり、デバイス固有の状態を永続化するための機能です。セッションを使用することで、UDID を毎回指定する必要がなくなり、ワークフローが簡潔になります。

**主な利点:**
- **マルチデバイスワークフロー**: iOS と Android を同時にテスト
- **UDID 省略**: セッションに紐づいたデバイスを自動使用
- **状態の永続化**: 最後のスナップショットをセッションに保存
- **並行実行**: 複数セッションで並列テスト実行

## Lifecycle

### 1. Create

セッションを作成し、デバイス UDID とプラットフォームを紐付けます。

```bash
agent-mobile session create <name> --udid <udid> -p <platform>
```

**例:**
```bash
# iOS シミュレータ
agent-mobile session create ios-dev --udid "ABC-123-DEF" -p ios

# Android エミュレータ
agent-mobile session create android-dev --udid "emulator-5554" -p android

# 実機
agent-mobile session create iphone15 --udid "00008030-001234567890ABCD" -p ios
```

### 2. Use

セッションを使用してコマンドを実行します。

**方法1: --session フラグ**
```bash
agent-mobile --session ios-dev snapshot
agent-mobile --session ios-dev tap @e1
```

**方法2: 環境変数**
```bash
export AGENT_MOBILE_SESSION=ios-dev
agent-mobile snapshot
agent-mobile tap @e1
```

### 3. List

すべての アクティブなセッションを一覧表示します。

```bash
# テキスト形式
agent-mobile session list

# 出力例:
# NAME            UDID                                     PLATFORM APP
# -------------------------------------------------------------------------------
# ios-dev         ABC-123-DEF                              ios      com.example.app
# android-dev     emulator-5554                            android  -

# JSON 形式
agent-mobile session list -f json
```

### 4. Show

現在のセッション情報を表示します。

```bash
agent-mobile --session ios-dev session show

# 出力例:
# Session: ios-dev
#   UDID: ABC-123-DEF
#   Platform: ios
#   App: com.example.app
#   Created: 2026-01-23T10:00:00Z
#   Last Activity: 2026-01-23T10:15:00Z
#   Last Snapshot:
#     ID: snap_abc123
#     Refs: 15
#     Time: 2026-01-23T10:15:00Z
```

### 5. Destroy

セッションを削除します。

```bash
agent-mobile session rm ios-dev
# 出力: Session 'ios-dev' removed
```

## Session Data

セッションには以下の情報が保存されます:

```json
{
  "name": "ios-dev",
  "udid": "ABC-123-DEF",
  "platform": "ios",
  "app": "com.example.app",
  "created_at": "2026-01-23T10:00:00Z",
  "last_activity": "2026-01-23T10:15:00Z",
  "last_snapshot": {
    "snapshot_id": "snap_abc123",
    "timestamp": "2026-01-23T10:15:00Z",
    "ref_count": 15
  }
}
```

**フィールド説明:**
- `name`: セッション名（一意）
- `udid`: デバイス UDID
- `platform`: ios または android
- `app`: 最後に起動したアプリの bundle ID
- `created_at`: セッション作成日時
- `last_activity`: 最後のアクティビティ日時
- `last_snapshot`: 最後に取得したスナップショット情報

## Storage Location

セッション情報は以下の場所に保存されます:

```bash
~/.agent-mobile/sessions/<session-name>.json
```

**例:**
```bash
~/.agent-mobile/sessions/ios-dev.json
~/.agent-mobile/sessions/android-dev.json
```

## Common Patterns

### Pattern 1: Single Device Development

開発中に1つのデバイスを継続的に使用する場合:

```bash
# セッション作成（一度だけ）
agent-mobile session create dev --udid "ABC-123" -p ios

# 環境変数設定（.bashrc や .zshrc に追加）
export AGENT_MOBILE_SESSION=dev

# 以降、すべてのコマンドで自動使用
agent-mobile snapshot
agent-mobile app launch com.example.app
agent-mobile tap @e1
```

### Pattern 2: Multi-Device Testing

iOS と Android で同じテストを並行実行:

```bash
# セッション作成
agent-mobile session create ios16 --udid "ABC-123" -p ios
agent-mobile session create android13 --udid "emulator-5554" -p android

# 並行実行
agent-mobile --session ios16 app launch com.example.app &
agent-mobile --session android13 app launch com.example.app &
wait

# 両方でスナップショット取得
agent-mobile --session ios16 snapshot -o ios16_snapshot.txt &
agent-mobile --session android13 snapshot -o android13_snapshot.txt &
wait

# 同じテストを両方で実行
for session in ios16 android13; do
  agent-mobile --session $session find text "Login" tap
  agent-mobile --session $session screenshot -o ${session}_result.png
done
```

### Pattern 3: Session Switching

複数のデバイス間を切り替えながら作業:

```bash
# iPhone 15 でテスト
export AGENT_MOBILE_SESSION=iphone15
agent-mobile snapshot
agent-mobile tap @e1

# iPad でテスト
export AGENT_MOBILE_SESSION=ipad
agent-mobile snapshot
agent-mobile tap @e1

# 元に戻す
export AGENT_MOBILE_SESSION=iphone15
```

### Pattern 4: CI/CD Integration

継続的インテグレーション環境での使用:

```bash
#!/bin/bash
# ci-test.sh

# セッション作成（一時的）
SESSION_NAME="ci-${CI_JOB_ID}"
agent-mobile session create "$SESSION_NAME" --udid "$DEVICE_UDID" -p ios

# テスト実行
export AGENT_MOBILE_SESSION="$SESSION_NAME"
agent-mobile app install "$APP_PATH"
agent-mobile app launch "$BUNDLE_ID"
agent-mobile snapshot
# ... テスト処理 ...

# クリーンアップ
agent-mobile session rm "$SESSION_NAME"
```

## Multi-Session Workflows

### Parallel Execution

複数のセッションで並列実行してテスト時間を短縮:

```bash
#!/bin/bash

# セッション作成
SESSIONS=("ios15" "ios16" "ios17")
for session in "${SESSIONS[@]}"; do
  agent-mobile session create "$session" --udid "${session}-udid" -p ios
done

# 並列実行関数
run_test() {
  local session=$1
  echo "Testing on $session..."
  agent-mobile --session "$session" app launch com.example.app
  agent-mobile --session "$session" snapshot
  agent-mobile --session "$session" find text "Login" tap
  agent-mobile --session "$session" screenshot -o "${session}_result.png"
}

# バックグラウンドで実行
for session in "${SESSIONS[@]}"; do
  run_test "$session" &
done

# すべて完了を待つ
wait
echo "All tests completed"
```

### Sequential Testing

デバイスごとに順次テスト:

```bash
#!/bin/bash

SESSIONS=("ios-dev" "android-dev")

for session in "${SESSIONS[@]}"; do
  echo "=== Testing on $session ==="
  export AGENT_MOBILE_SESSION="$session"

  agent-mobile app launch com.example.app
  sleep 2
  agent-mobile snapshot -i
  agent-mobile find label "Email" fill "test@example.com"
  agent-mobile find label "Password" fill "password123"
  agent-mobile find text "Login" tap
  sleep 2
  agent-mobile screenshot -o "${session}_login.png"

  echo "✓ $session test completed"
done
```

## Session State Management

### Last Snapshot Persistence

セッションには最後のスナップショットが保存されます（同一プロセス内）:

```bash
# セッション使用
export AGENT_MOBILE_SESSION=dev

# スナップショット取得（セッションに保存）
agent-mobile snapshot

# 後続コマンドで要素参照を使用可能
agent-mobile tap @e1  # セッションからスナップショットを読み込む
agent-mobile fill @e2 "text"
```

**注意:**
別プロセスからは参照を使用できません（現在の制限）:

```bash
# プロセス1
export AGENT_MOBILE_SESSION=dev
agent-mobile snapshot
# @e1, @e2, ... が生成される

# プロセス2（別ターミナル）
export AGENT_MOBILE_SESSION=dev
agent-mobile tap @e1  # エラー: スナップショット情報なし
```

### App Tracking

最後に起動したアプリが記録されます:

```bash
agent-mobile --session dev app launch com.example.app
agent-mobile --session dev session show
# App: com.example.app が表示される
```

### Activity Timestamps

セッションのアクティビティは自動的に記録されます:

- `created_at`: セッション作成時刻
- `last_activity`: 最後にセッションが使用された時刻

```bash
agent-mobile session list -f json | jq '.[] | {name, last_activity}'
```

## Best Practices

### 1. 命名規則

わかりやすいセッション名を使用:

```bash
# 良い例
agent-mobile session create ios16-iphone14-pro --udid ... -p ios
agent-mobile session create android13-pixel7 --udid ... -p android
agent-mobile session create ci-staging --udid ... -p ios

# 悪い例
agent-mobile session create s1 --udid ... -p ios  # 不明確
agent-mobile session create test --udid ... -p ios  # 汎用的すぎ
```

### 2. セッションのクリーンアップ

使わなくなったセッションは削除:

```bash
# 一覧確認
agent-mobile session list

# 不要なセッションを削除
agent-mobile session rm old-session
```

### 3. 環境変数の活用

`.bashrc` や `.zshrc` にデフォルトセッションを設定:

```bash
# ~/.zshrc
export AGENT_MOBILE_SESSION=dev
```

これにより、`--session` フラグを毎回指定する必要がなくなります。

### 4. セッションごとの設定分離

プロジェクトごとにセッションを分ける:

```bash
# プロジェクトA
agent-mobile session create project-a-ios --udid ... -p ios
agent-mobile session create project-a-android --udid ... -p android

# プロジェクトB
agent-mobile session create project-b-ios --udid ... -p ios
```

### 5. CI/CD での一時セッション

CI/CD では一時的なセッションを使用し、完了後に削除:

```bash
SESSION_NAME="ci-${BUILD_NUMBER}"
agent-mobile session create "$SESSION_NAME" --udid "$UDID" -p ios
# ... テスト実行 ...
agent-mobile session rm "$SESSION_NAME"
```

## Limitations

### 1. 同一プロセス内のみ

現在、スナップショット情報は同一プロセス内でのみ共有されます:

```bash
# 同じプロセス（OK）
agent-mobile --session dev snapshot
agent-mobile --session dev tap @e1  # OK

# 別プロセス（NG）
# ターミナル1
agent-mobile --session dev snapshot

# ターミナル2
agent-mobile --session dev tap @e1  # エラー: スナップショットなし
```

**回避策:**
各プロセスで `snapshot` を取得してください。

### 2. セッション間の状態共有なし

異なるセッション間でスナップショットは共有されません:

```bash
agent-mobile --session ios snapshot
agent-mobile --session android tap @e1  # エラー: 別セッション
```

### 3. ファイルシステムベース

セッション情報はローカルファイルシステムに保存されるため、異なるマシン間では共有されません。

## Troubleshooting

### "Session 'xxx' not found"

**原因:**
セッションが作成されていない。

**解決:**
```bash
# セッション一覧確認
agent-mobile session list

# セッション作成
agent-mobile session create xxx --udid ... -p ios
```

### "Session 'xxx' already exists"

**原因:**
同名のセッションが既に存在する。

**解決:**
```bash
# 既存セッションを削除
agent-mobile session rm xxx

# または別の名前を使用
agent-mobile session create xxx-new --udid ... -p ios
```

### セッションが更新されない

**原因:**
セッション情報のキャッシュ問題。

**解決:**
```bash
# セッションを作り直す
agent-mobile session rm old-session
agent-mobile session create old-session --udid ... -p ios
```

## Advanced Use Cases

### Dynamic Session Creation

デバイス一覧から自動的にセッションを作成:

```bash
#!/bin/bash

# iOS デバイス一覧を取得
devices=$(agent-mobile device list -p ios -f json)

# 各デバイスにセッションを作成
echo "$devices" | jq -r '.[] | "\(.name) \(.udid)"' | while read -r name udid; do
  session_name=$(echo "$name" | tr ' ' '-' | tr '[:upper:]' '[:lower:]')
  agent-mobile session create "$session_name" --udid "$udid" -p ios
  echo "Created session: $session_name"
done
```

### Session-Based Test Distribution

セッションを使ってテストを分散実行:

```bash
#!/bin/bash

SESSIONS=("device1" "device2" "device3")
TESTS=("test1.sh" "test2.sh" "test3.sh")

# テストを各セッションに割り当て
for i in "${!SESSIONS[@]}"; do
  session="${SESSIONS[$i]}"
  test="${TESTS[$i]}"

  echo "Running $test on $session..."
  AGENT_MOBILE_SESSION="$session" bash "$test" &
done

wait
echo "All tests completed"
```

### Session Monitoring

セッションのアクティビティを監視:

```bash
#!/bin/bash

while true; do
  clear
  echo "=== Session Monitor ==="
  agent-mobile session list
  sleep 5
done
```

## Related Concepts

- [Element References](element-references.md) - セッションとスナップショットの関係
- [Multi-Device Testing](../SKILL.md#common-workflows) - マルチデバイステストのワークフロー
- [SKILL.md - Session Management](../SKILL.md#session-management) - セッション管理コマンドリファレンス

---

**Last Updated**: 2026-01-23
