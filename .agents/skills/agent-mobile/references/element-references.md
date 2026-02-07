# Element References (@e1, @e2, ...)

agent-mobile の要素参照システムの完全ガイド。

## Overview

Element References（要素参照）は、UI 要素に対する短縮 ID システムです。`@e1`, `@e2`, `@e3` のような形式で、トークン使用量を劇的に削減しながら要素を参照できます。

**なぜ重要なのか:**
- **トークン効率**: 長いセマンティックロケーターの代わりに短い参照を使用
- **高速**: スナップショット取得後、繰り返し検索不要
- **明確**: 各要素に一意の ID が付与される

## How It Works

### 1. スナップショット取得

`agent-mobile snapshot` コマンドを実行すると、現在の UI 階層がキャプチャされ、各インタラクティブ要素に `@e1`, `@e2`, ... の参照が割り当てられます。

```bash
agent-mobile snapshot

# 出力例:
# @e1 Button "Login" (enabled)
#   frame: (100.0, 200.0, 80.0, 44.0)
# @e2 TextField "Email" (enabled)
#   placeholder: "Enter your email"
#   frame: (50.0, 100.0, 200.0, 40.0)
# @e3 SecureTextField "Password" (enabled)
#   placeholder: "Enter your password"
#   frame: (50.0, 150.0, 200.0, 40.0)
```

### 2. 参照の使用

生成された参照を使って、各種コマンドで要素を操作します。

```bash
# タップ
agent-mobile tap @e1

# テキスト入力
agent-mobile fill @e2 "user@example.com"
agent-mobile fill @e3 "password123"

# プロパティ取得
agent-mobile get @e1 label
agent-mobile get @e2 value

# 状態チェック
agent-mobile is @e1 enabled
```

### 3. 参照の有効期限

参照は**最後に取得したスナップショットに紐づいています**。画面が変更されたら、新しいスナップショットを取得する必要があります。

```bash
# ログイン画面
agent-mobile snapshot
# @e1: Login Button
agent-mobile tap @e1

# ホーム画面に遷移（@e1 は無効）
sleep 2
agent-mobile snapshot  # 新しい参照を生成
# @e1: Logout Button (新しい要素)
# @e2: Settings Button
```

## Reference Format

### 命名規則

参照は `@e` プレフィックス + 連番で構成されます:
- `@e1` - 最初の要素
- `@e2` - 2番目の要素
- `@e10` - 10番目の要素
- `@e99` - 99番目の要素

### 割り当て順序

参照は以下の順序で割り当てられます:
1. **ビジュアル順序**: 画面上の表示順（上から下、左から右）
2. **インタラクティブ要素優先**: デフォルトではインタラクティブな要素のみ

```bash
# デフォルト（インタラクティブ要素のみ）
agent-mobile snapshot -i
# @e1: Button "Login"
# @e2: TextField "Email"
# @e3: SecureTextField "Password"

# すべての要素（非インタラクティブも含む）
agent-mobile snapshot
# @e1: StaticText "Welcome"
# @e2: StaticText "Please login to continue"
# @e3: TextField "Email"
# @e4: SecureTextField "Password"
# @e5: Button "Login"
```

## Supported Commands

要素参照は以下のコマンドで使用できます:

### Navigation & Interaction

```bash
# Tap
agent-mobile tap @e1
agent-mobile tap @e2

# Long press
agent-mobile long-press @e1
agent-mobile long-press @e1 --duration 2.0

# Fill (clear + type)
agent-mobile fill @e2 "text"

# Swipe (要素内)
agent-mobile swipe up @e1

# Scroll (要素内)
agent-mobile scroll down --element @e1
```

### Element Discovery

```bash
# Get property
agent-mobile get @e1 text
agent-mobile get @e1 label
agent-mobile get @e1 value
agent-mobile get @e1 placeholder

# Is condition
agent-mobile is @e1 enabled
agent-mobile is @e1 disabled

# Wait
agent-mobile wait @e1
agent-mobile wait @e1 --timeout 10
```

## Advantages

### 1. Token Efficiency

**セマンティックロケーターとの比較:**

❌ **トークン浪費** (約120 tokens):
```bash
agent-mobile find label "Email" fill "user@example.com"
agent-mobile find label "Password" fill "password123"
agent-mobile find text "Login" tap
```

✅ **トークン効率的** (約30 tokens):
```bash
agent-mobile snapshot  # 一度だけ
agent-mobile fill @e2 "user@example.com"
agent-mobile fill @e3 "password123"
agent-mobile tap @e1
```

**削減率: 約75%**

### 2. Speed

スナップショットは一度だけ取得すればよいため、繰り返し検索が不要です。

```bash
# セマンティックロケーター: 毎回 accessibility_info を取得
agent-mobile find text "Button1" tap  # ~500ms
agent-mobile find text "Button2" tap  # ~500ms
agent-mobile find text "Button3" tap  # ~500ms
# 合計: ~1500ms

# 要素参照: 一度だけ取得
agent-mobile snapshot  # ~500ms
agent-mobile tap @e1   # ~100ms
agent-mobile tap @e2   # ~100ms
agent-mobile tap @e3   # ~100ms
# 合計: ~800ms (約50% 高速)
```

### 3. Clarity

各要素に一意の ID が付与されるため、コードが読みやすくなります。

```bash
# どのボタンをタップしているか明確
agent-mobile tap @e1  # Login Button
agent-mobile tap @e2  # Submit Button
```

## Limitations

### 1. スナップショットへの依存

参照は最後に取得したスナップショットに紐づいています。画面が変わると無効になります。

❌ **悪い例:**
```bash
agent-mobile snapshot
agent-mobile tap @e1  # ページ遷移
# @e1 は無効（新しい画面の要素を指していない）
agent-mobile tap @e1  # エラー!
```

✅ **良い例:**
```bash
agent-mobile snapshot
agent-mobile tap @e1  # ページ遷移
sleep 1
agent-mobile snapshot  # 新しいスナップショット
agent-mobile tap @e1   # 新しい画面の @e1
```

### 2. 画面外要素の非対応

現在の実装では、画面外（スクロール外）の要素には参照が付与されません。

```bash
# デフォルト: スクロールして全要素を取得
agent-mobile snapshot

# スクロール無効: 画面内のみ
agent-mobile snapshot --no-scroll
```

**将来の改善:**
スクロールによる自動要素収集は現在サポートされています（デフォルトで有効）。

### 3. セッション非依存

参照はセッションをまたいで永続化しません（現在の制限）。

```bash
# Session 1
agent-mobile --session ios snapshot
# @e1: Login Button

# Session 2 (別プロセス)
agent-mobile --session ios tap @e1  # エラー（スナップショット情報なし）
```

**回避策:**
セッション内では最後のスナップショットが保存されるため、同一プロセス内では参照を再利用できます。

### 4. 動的 UI の問題

動的に生成される UI では、同じ要素でも参照番号が変わる可能性があります。

```bash
# 最初のスナップショット
agent-mobile snapshot
# @e1: Item "Apple"
# @e2: Item "Banana"
# @e3: Item "Cherry"

# "Banana" を削除後
agent-mobile snapshot
# @e1: Item "Apple"
# @e2: Item "Cherry"  # 番号が変わった!
```

**ベストプラクティス:**
動的 UI では、画面変更後に必ずスナップショットを再取得してください。

## Best Practices

### 1. スナップショットの最適なタイミング

**画面遷移後は必ず再取得:**
```bash
agent-mobile snapshot
agent-mobile tap @e1  # ページ遷移
sleep 1              # アニメーション待機
agent-mobile snapshot  # 新しいスナップショット
```

**フォーム入力では再取得不要:**
```bash
agent-mobile snapshot
agent-mobile fill @e1 "text1"
agent-mobile fill @e2 "text2"  # スナップショット不要
agent-mobile fill @e3 "text3"  # スナップショット不要
agent-mobile tap @e4           # スナップショット不要
```

### 2. インタラクティブ要素のフィルタリング

大きなアプリでは `-i` オプションを使用してインタラクティブ要素のみを表示します。

```bash
# デフォルト（すべての要素）
agent-mobile snapshot
# @e1: StaticText "Title"
# @e2: StaticText "Description"
# @e3: Button "Click Me"
# @e4: StaticText "Footer"
# ... 100+ elements

# インタラクティブ要素のみ
agent-mobile snapshot -i
# @e1: Button "Click Me"
# @e2: TextField "Input"
# @e3: Button "Submit"
# ... 10 elements (90% 削減)
```

### 3. コンパクト表示とデプス制限

さらに絞り込むには `-c` (compact) と `-d` (depth) を使用します。

```bash
# 空要素を削除 + 深さ3まで
agent-mobile snapshot -i -c -d 3
# @e1: Button "Login"
# @e2: TextField "Email"
# @e3: TextField "Password"
```

### 4. JSON 出力で詳細情報を取得

プログラムで処理する場合は JSON 形式を使用します。

```bash
agent-mobile snapshot -f json > snapshot.json

# jq で解析
cat snapshot.json | jq '.elements[] | select(.element_type == "Button")'
```

### 5. スナップショットのファイル保存

後で参照できるようにファイルに保存します。

```bash
# テキスト形式
agent-mobile snapshot -o current_ui.txt

# JSON 形式
agent-mobile snapshot -o current_ui.json -f json

# 後で find コマンドで使用
agent-mobile find type Button --snapshot current_ui.json tap
```

## Advanced Patterns

### 1. 参照の抽出

スナップショット出力から参照を抽出して変数に保存:

```bash
# スナップショット取得
agent-mobile snapshot -i > /tmp/snap.txt

# grep で特定の要素を探す
LOGIN_BTN=$(grep "Login" /tmp/snap.txt | grep -o "@e[0-9]*" | head -1)
EMAIL_FIELD=$(grep "Email" /tmp/snap.txt | grep -o "@e[0-9]*" | head -1)

# 変数を使用
agent-mobile fill $EMAIL_FIELD "user@example.com"
agent-mobile tap $LOGIN_BTN
```

### 2. ループ処理

複数の要素を順番に操作:

```bash
agent-mobile snapshot -i

# @e1 から @e5 までタップ
for i in {1..5}; do
  agent-mobile tap @e$i
  sleep 0.5
done
```

### 3. 条件付き操作

特定の要素が有効な場合のみ操作:

```bash
agent-mobile snapshot

if agent-mobile is @e1 enabled; then
  agent-mobile tap @e1
  echo "Button tapped"
else
  echo "Button is disabled"
fi
```

### 4. バッチ操作

複数の入力を一度に処理:

```bash
# フォームデータ
declare -A form_data=(
  ["@e1"]="John"
  ["@e2"]="Doe"
  ["@e3"]="john@example.com"
  ["@e4"]="555-1234"
)

# スナップショット取得
agent-mobile snapshot -i

# 一括入力
for ref in "${!form_data[@]}"; do
  agent-mobile fill "$ref" "${form_data[$ref]}"
done
```

## Error Messages

### "Element ref @eN not found"

**原因:**
スナップショット取得後に画面が変更されたか、そもそもその参照が存在しない。

**解決策:**
```bash
# 新しいスナップショットを取得
agent-mobile snapshot

# 参照が存在するか確認
agent-mobile snapshot | grep "@e10"
```

### "No snapshot available"

**原因:**
スナップショットを取得せずに参照を使用した。

**解決策:**
```bash
# 必ずスナップショットを先に取得
agent-mobile snapshot
agent-mobile tap @e1
```

## Comparison: Semantic Locators vs Element References

| Feature | Semantic Locators | Element References |
|---------|-------------------|-------------------|
| **可読性** | 高い (`find text "Login"`) | 中程度 (`tap @e1`) |
| **トークン効率** | 低い (~30 tokens/command) | 高い (~10 tokens/command) |
| **速度** | 遅い（毎回検索） | 速い（検索不要） |
| **動的 UI** | 対応しやすい | 再スナップショット必要 |
| **初回実行** | スナップショット不要 | スナップショット必要 |

**使い分けガイドライン:**

**Semantic Locators を使う場合:**
- 初回の要素検索
- 可読性が重要なコード
- ワンライナーでの実行
- 動的に変わる UI

**Element References を使う場合:**
- 繰り返し操作（ループ、バッチ処理）
- トークン効率が重要
- 高速実行が必要
- 固定的な UI

**ハイブリッドアプローチ（推奨）:**
```bash
# 1. Semantic Locator で初回検索
agent-mobile find text "Login" tap

# 2. 次の画面で snapshot + 参照使用
agent-mobile snapshot -i
agent-mobile fill @e1 "user@example.com"
agent-mobile fill @e2 "password123"
agent-mobile tap @e3
```

## Session Integration

セッションを使用すると、最後のスナップショットが保存されます（同一プロセス内）。

```bash
# セッション作成
export AGENT_MOBILE_SESSION=test

# スナップショット取得（セッションに保存される）
agent-mobile snapshot

# 後続コマンドで参照を使用可能
agent-mobile tap @e1  # セッションからスナップショットを読み込む
```

**注意:**
別プロセスからは参照を使用できません（将来の改善予定）。

## Future Enhancements

以下の機能は現在計画中です:

1. **永続的な参照**: セッションをまたいで参照を保存
2. **スマート参照**: 画面変更後も同じ要素を追跡
3. **名前付き参照**: `@login_button` のようなカスタム名
4. **参照のエイリアス**: 複数の名前で同じ要素を参照

## Related Concepts

- [Semantic Locators](semantic-locators.md) - セマンティックロケーターの詳細
- [Snapshot Command](../SKILL.md#element-discovery) - スナップショット取得の詳細
- [Session Management](session-management.md) - セッション管理

---

**Last Updated**: 2026-01-23
