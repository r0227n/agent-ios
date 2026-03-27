# Semantic Locators

agent-mobile のセマンティックロケーターシステムの完全ガイド。

## Overview

セマンティックロケーターは、UI 要素を人間が理解しやすい属性（タイプ、テキスト、ラベル等）で検索する仕組みです。XPath や複雑なセレクタを使わずに、自然な記述で要素を見つけることができます。

## Locator Types

### 1. Type Locator

要素のタイプ（Button, TextField, Image など）で検索します。

```bash
# すべてのボタンを検索
agent-mobile find type Button

# 最初のテキストフィールド
agent-mobile find type TextField --first

# 2番目の画像
agent-mobile find type Image --nth 1
```

**よく使われる要素タイプ:**

**iOS:**
- `Button` - ボタン
- `TextField` - テキスト入力フィールド
- `SecureTextField` - パスワード入力フィールド
- `StaticText` - 静的テキスト（ラベル）
- `Image` - 画像
- `Cell` - テーブルセル
- `Switch` - トグルスイッチ
- `Slider` - スライダー
- `TabBar` - タブバー
- `NavigationBar` - ナビゲーションバー

**Android:**
- `Button` - ボタン
- `EditText` - テキスト入力フィールド
- `TextView` - テキストビュー
- `ImageView` - 画像ビュー
- `CheckBox` - チェックボックス
- `RadioButton` - ラジオボタン
- `Switch` - スイッチ
- `SeekBar` - シークバー

**大文字小文字の区別:**
タイプ名は大文字小文字を区別しません。`Button`、`button`、`BUTTON` はすべて同じです。

### 2. Text Locator

要素のテキスト内容（label または value）で検索します。

```bash
# 部分一致（デフォルト）
agent-mobile find text "Login"      # "Login", "Login Now", "User Login" などにマッチ

# 完全一致
agent-mobile find text "Login" --exact  # "Login" のみにマッチ
```

**マッチング対象:**
- `label`: 要素のラベル（iOS: AXLabel、Android: content-desc/text）
- `value`: 要素の値（iOS: AXValue、Android: text）

**ユースケース:**
```bash
# ボタンテキストで検索
agent-mobile find text "Submit" tap

# ログインボタンを探してタップ
agent-mobile find text "Log in" tap
agent-mobile find text "Sign in" tap

# 特定のテキストを含むセルを探す
agent-mobile find text "Settings" --nth 0
```

### 3. Label Locator

要素のラベル属性のみで検索します（text locator よりも限定的）。

```bash
# 部分一致
agent-mobile find label "Email"

# 完全一致
agent-mobile find label "Email Address" --exact
```

**Text vs Label の違い:**
- `text`: label **または** value を検索
- `label`: label **のみ**を検索

**ユースケース:**
```bash
# テキストフィールドのラベルで検索
agent-mobile find label "Username" fill "john_doe"
agent-mobile find label "Password" fill "secret123"

# ナビゲーション項目を探す
agent-mobile find label "Back" tap
```

### 4. Placeholder Locator

テキストフィールドのプレースホルダーテキストで検索します。

```bash
# 部分一致
agent-mobile find placeholder "email"  # "Enter email", "Email address" などにマッチ

# 完全一致
agent-mobile find placeholder "Enter your email" --exact
```

**ユースケース:**
```bash
# プレースホルダーで入力フィールドを特定
agent-mobile find placeholder "Search" fill "query text"
agent-mobile find placeholder "Enter password" fill "secret"
```

**注意:**
- プレースホルダーは TextField/EditText にのみ存在します
- 入力後はプレースホルダーが消えることがあります

### 5. State Locators

要素の状態（有効/無効）で検索します。

```bash
# 有効な要素すべて
agent-mobile find enabled

# 無効な要素すべて
agent-mobile find disabled

# 最初の有効なボタン
agent-mobile find enabled --first
```

**ユースケース:**
```bash
# 有効なボタンのみをタップ
agent-mobile find enabled tap

# 無効なボタンの数を数える
agent-mobile find disabled --all -f json | jq length
```

## Position Specifiers

複数の要素がマッチした場合、位置を指定して選択します。

### `--first` (デフォルト)

最初の要素を選択します。

```bash
agent-mobile find type Button --first  # 明示的
agent-mobile find type Button          # 暗黙的（デフォルト）
```

### `--last`

最後の要素を選択します。

```bash
# 最後のボタン
agent-mobile find type Button --last

# ページの一番下のセルをタップ
agent-mobile find type Cell --last tap
```

### `--nth N`

N 番目の要素を選択します（0-indexed）。

```bash
# 0番目（最初）
agent-mobile find type Button --nth 0

# 1番目（2つ目）
agent-mobile find type Button --nth 1

# 2番目（3つ目）
agent-mobile find type Button --nth 2
```

**範囲外エラー:**
```bash
agent-mobile find type Button --nth 10
# エラー: "Index 10 out of range. Found 3 element(s)."
```

### `--all`

すべてのマッチング要素を返します（アクションなし時のみ）。

```bash
# すべてのボタンを JSON で取得
agent-mobile find type Button --all -f json

# すべてのテキストフィールドの ref を取得
agent-mobile find type TextField --all
```

**制約:**
```bash
# エラー: --all とアクションは併用不可
agent-mobile find type Button --all tap
# エラー: "--all cannot be used with an action (tap, fill, etc.)"
```

## Matching Modes

### 部分一致（デフォルト）

大文字小文字を区別せず、部分一致します。

```bash
agent-mobile find text "log"
# マッチ: "Login", "logout", "Catalog", "LOG OUT"
```

### 完全一致（`--exact`）

大文字小文字を区別せず、完全一致します。

```bash
agent-mobile find text "Login" --exact
# マッチ: "Login", "login", "LOGIN"
# 非マッチ: "Login Now", "User Login"
```

**適用可能なロケーター:**
- `text`
- `label`
- `placeholder`

**適用不可:**
- `type` (常に完全一致)
- `enabled`/`disabled` (状態判定のみ)

## Actions

find コマンドに直接アクションを指定できます。

### `tap`

要素をタップします。

```bash
agent-mobile find text "Login" tap
agent-mobile find type Button --nth 1 tap
```

### `long-press`

要素を長押しします。

```bash
agent-mobile find text "Delete" long-press
agent-mobile find type Cell --first long-press
```

### `fill <text>`

テキストフィールドをクリアして入力します。

```bash
agent-mobile find label "Email" fill "user@example.com"
agent-mobile find placeholder "Password" fill "secret123"
```

**動作:**
1. 要素をタップしてフォーカス
2. 既存のテキストを削除（最大50文字）
3. 新しいテキストを入力

### `clear`

テキストフィールドをクリアします。

```bash
agent-mobile find label "Search" clear
```

## Combining Locators and Actions

### パターン1: 検索のみ

```bash
# ref を返す
agent-mobile find text "Login"
# 出力: @e1

# JSON で詳細情報を返す
agent-mobile find text "Login" -f json
# 出力: {"ref": "@e1", "type": "Button", "label": "Login", ...}
```

### パターン2: 検索 + アクション

```bash
# ワンライナーで実行
agent-mobile find text "Login" tap

# fillアクションには値が必要
agent-mobile find label "Email" fill "test@example.com"
```

### パターン3: 複数要素 + 位置指定 + アクション

```bash
# 2番目のボタンをタップ
agent-mobile find type Button --nth 1 tap

# 最後のセルを長押し
agent-mobile find type Cell --last long-press
```

## Advanced Patterns

### 1. チェーン操作

```bash
# 検索して ref を保存、その後使用
REF=$(agent-mobile find text "Login")
agent-mobile tap $REF
```

### 2. 条件分岐

```bash
# 要素の存在チェック
if agent-mobile find text "Logout" 2>/dev/null; then
  echo "User is logged in"
  agent-mobile find text "Logout" tap
else
  echo "User is not logged in"
fi
```

### 3. ループ処理

```bash
# すべてのセルを順番にタップ
for i in {0..4}; do
  agent-mobile find type Cell --nth $i tap
  sleep 1
  agent-mobile swipe left
done
```

### 4. 複雑なフォーム入力

```bash
# スナップショットを取得
agent-mobile snapshot -i > /tmp/form.txt

# セマンティックロケーターで順次入力
agent-mobile find label "First Name" fill "John"
agent-mobile find label "Last Name" fill "Doe"
agent-mobile find label "Email" fill "john@example.com"
agent-mobile find label "Phone" fill "555-1234"
agent-mobile find text "Submit" tap
```

## Performance Tips

### 1. スナップショットの再利用

❌ **非効率:**
```bash
agent-mobile find text "Button1" tap
agent-mobile find text "Button2" tap  # 内部で snapshot 再取得
agent-mobile find text "Button3" tap  # 内部で snapshot 再取得
```

✅ **効率的:**
```bash
agent-mobile snapshot
# @e1: Button1
# @e2: Button2
# @e3: Button3
agent-mobile tap @e1
agent-mobile tap @e2
agent-mobile tap @e3
```

### 2. 位置指定の活用

```bash
# 最初の要素なら --first は省略可能
agent-mobile find type Button        # OK
agent-mobile find type Button --first # 冗長
```

### 3. 型の限定

```bash
# 広すぎる検索
agent-mobile find text "Next"  # Button, StaticText, Cell など多数ヒット

# 型を限定
agent-mobile find type Button --first  # 最初のボタンのみ
```

## Error Messages

### "No elements found matching..."

**原因:**
セマンティックロケーターがどの要素にもマッチしなかった。

**解決策:**
```bash
# UI 状態を確認
agent-mobile snapshot -i

# 部分一致を試す（--exact を外す）
agent-mobile find text "Login"  # "Login Now" にもマッチ

# 大文字小文字を確認（自動で無視されるはず）
agent-mobile find text "login"  # "Login" にマッチ
```

### "Index N out of range"

**原因:**
`--nth N` で指定した N が要素数を超えている。

**解決策:**
```bash
# すべての要素を確認
agent-mobile find type Button --all -f json

# 最後の要素を選択
agent-mobile find type Button --last
```

### "--all cannot be used with an action"

**原因:**
`--all` とアクション（tap, fill など）を併用した。

**解決策:**
```bash
# --all を削除
agent-mobile find type Button tap  # 最初の要素をタップ

# または位置指定を使用
agent-mobile find type Button --nth 2 tap
```

## Comparison with Other Approaches

### vs XPath

| XPath | Semantic Locator |
|-------|------------------|
| `//XCUIElementTypeButton[@label="Login"]` | `find text "Login"` |
| `(//XCUIElementTypeTextField)[2]` | `find type TextField --nth 1` |
| `//XCUIElementTypeButton[@enabled="true"]` | `find enabled` |

**利点:**
- 読みやすい
- プラットフォーム非依存
- 部分一致がデフォルト

### vs Element References

| Element Reference | Semantic Locator |
|-------------------|------------------|
| `tap @e1` | `find text "Login" tap` |
| (事前に snapshot 必要) | (snapshot 不要) |
| トークン効率的 | 可読性が高い |

**使い分け:**
- **Semantic Locator**: 初回検索、可読性重視
- **Element Reference**: 繰り返し操作、トークン効率重視

## Related Concepts

- [Element References](element-references.md) - @e1 参照システム
- [Snapshot Command](../SKILL.md#element-discovery) - スナップショット取得
- [Find Command](../SKILL.md#element-discovery) - find コマンド詳細

---

**Last Updated**: 2026-01-23
