# agent-mobile 実装TODO

agent-browserとの機能比較に基づく未実装機能リスト。
参照: https://github.com/vercel-labs/agent-browser

## 注意事項
- ブラウザ特有の操作（hover, mouse, cookies, localStorage, tab, window等）は除外
- ドラッグ&ドロップは既にswipeとして実装済み

## 凡例
- 🔴 優先度: 高（すぐに実装すべき）
- 🟡 優先度: 中（検討すべき）

---

## 1. 要素操作 (Element Actions)

### 🔴 select - Picker/ドロップダウン選択

**agent-browserでの仕様:**
```bash
agent-browser select <selector> <value>
```

**モバイル実装方針:**
- **iOS**: UIPickerViewの選択（WheelChangedイベント）
- **Android**: Spinner/DropDownMenuの選択（adb shell input tap + swipe）

**技術的検討事項:**
- gRPC RPC: なし（HIDイベントで実装）
- iOS: `hid()` RPC でタップ + スワイプシーケンス
- Android: adb shell input でタップ + テキスト設定

**参考実装:**
- agent-mobile: `src/core/find.rs` (tap実装)
- agent-mobile: `src/platform/ios/grpc/hid.rs` (HIDイベント送信)

**実装例:**
```rust
// src/core/select.rs (新規)
pub async fn select(selector: &str, value: &str) -> Result<()> {
    // 1. 要素を検索（Picker/Spinner）
    // 2. タップして開く
    // 3. 値を選択（スワイプまたはタップ）
    // 4. 確定（iOS: Done, Android: 自動）
}
```

---

### 🔴 check/uncheck - チェックボックス/スイッチ操作

**agent-browserでの仕様:**
```bash
agent-browser check <selector>
agent-browser uncheck <selector>
```

**モバイル実装方針:**
- **iOS**: UISwitchのトグル、UICheckBoxのタップ
- **Android**: Switch/CheckBoxのタップ

**技術的検討事項:**
- gRPC RPC: `hid()` RPCでタップ実装
- 状態確認: accessibility_info で現在の値取得（AXValue: 0/1）
- 冪等性: 既に目的の状態なら操作スキップ

**参考実装:**
- agent-mobile: `src/core/tap.rs` (タップ実装)
- agent-mobile: `src/snapshot/types.rs` (Element.enabled)

**実装例:**
```rust
// src/core/check.rs (新規)
pub async fn check(selector: &str, should_check: bool) -> Result<()> {
    // 1. 要素を検索（type=Switch/CheckBox）
    // 2. 現在の状態を取得（value属性）
    // 3. 目的の状態と異なればタップ
}
```

---

## 2. 情報取得 (Get Commands)

### 🔴 get attr - 属性取得

**agent-browserでの仕様:**
```bash
agent-browser get attr <selector> <attribute>
# 例: get attr "#email" placeholder
# 出力: Enter your email
```

**モバイル実装方針:**
- **iOS**: accessibility_infoから指定属性を抽出（AXLabel, AXValue, AXPlaceholderValue等）
- **Android**: uiautomator dumpからXML属性を抽出

**技術的検討事項:**
- gRPC RPC: `accessibility_info()` 使用
- 属性マッピング: iOS（AX*）⇔ Android（content-desc, text, resource-id）
- 既存実装: `get` コマンドに `--attr` オプション追加

**参考実装:**
- agent-mobile: `src/core/get.rs` (既存)
- agent-mobile: `src/snapshot/extractor/extractor_ios.rs` (属性抽出)

**実装例:**
```rust
// src/core/get.rs に追加
pub enum GetTarget {
    Text, Value, Label, Placeholder,
    Attr(String), // 新規: 任意属性
}
```

---

### 🔴 get count - マッチング要素数

**agent-browserでの仕様:**
```bash
agent-browser get count <selector>
# 出力: 5
```

**モバイル実装方針:**
- **共通**: find コマンドで `--all` 指定し、マッチ数をカウント

**技術的検討事項:**
- 既存実装: `find --all` で要素リスト取得可能
- 出力形式: 数値のみ（JSON: `{"count": 5}`）

**参考実装:**
- agent-mobile: `src/core/find.rs` (--all実装済み)

**実装例:**
```rust
// src/core/get.rs に追加
pub async fn get_count(selector: &str) -> Result<usize> {
    let elements = find_all(selector).await?;
    Ok(elements.len())
}
```

---

### 🔴 get box - バウンディングボックス

**agent-browserでの仕様:**
```bash
agent-browser get box <selector>
# 出力: {"x": 100, "y": 200, "width": 150, "height": 50}
```

**モバイル実装方針:**
- **共通**: accessibility_infoから `frame` 情報を取得（既に実装済み）

**技術的検討事項:**
- 既存実装: Element.frame に座標情報あり
- 本質的に `get frame` のエイリアス

**参考実装:**
- agent-mobile: `src/snapshot/types.rs` (Element.frame)

**実装例:**
```rust
// src/core/get.rs に追加
pub async fn get_box(selector: &str) -> Result<Frame> {
    let element = find_element(selector).await?;
    Ok(element.frame)
}
```

---

## 3. 状態確認 (Is Commands)

### 🔴 is visible - 可視性判定

**agent-browserでの仕様:**
```bash
agent-browser is visible <selector>
# exit code: 0 (visible) or 1 (not visible)
```

**モバイル実装方針:**
- **iOS**: frame座標が画面内 && enabled=true
- **Android**: visibility属性が "visible"

**技術的検討事項:**
- ビューポート判定: frame.y が画面高さ内
- 既存: `is` コマンドに `visible` バリアント追加

**参考実装:**
- agent-mobile: `src/core/is_cmd.rs` (既存)

**実装例:**
```rust
// src/core/is_cmd.rs に追加
pub enum IsCondition {
    Enabled, Disabled,
    Visible, // 新規
}
```

---

### 🔴 is checked - チェック状態判定

**agent-browserでの仕様:**
```bash
agent-browser is checked <selector>
# exit code: 0 (checked) or 1 (unchecked)
```

**モバイル実装方針:**
- **iOS**: AXValue=1 でチェック済み
- **Android**: checked属性="true"

**技術的検討事項:**
- Element.value: "0" or "1" で判定
- Switch/CheckBox/RadioButton が対象

**参考実装:**
- agent-mobile: `src/core/is_cmd.rs`

**実装例:**
```rust
// src/core/is_cmd.rs に追加
pub enum IsCondition {
    Enabled, Disabled,
    Visible,
    Checked, // 新規
}
```

---

## 4. 待機コマンド (Wait Commands)

### 🔴 wait --text - テキスト出現待機

**agent-browserでの仕様:**
```bash
agent-browser wait --text "Welcome"
# 指定テキストが出現するまで待機（タイムアウト付き）
```

**モバイル実装方針:**
- **共通**: ポーリングで `find text "Welcome"` を実行
- タイムアウト: デフォルト30秒

**技術的検討事項:**
- 既存: `wait <selector>` は要素待機
- 追加: `--text` オプションでテキスト待機

**現在の実装状況:**
- 🟡 **実装状況要確認** - src/core/wait.rs の詳細確認待ち
- WaitArgs に text フィールドが追加されているかを確認

**参考実装:**
- agent-mobile: `src/core/wait.rs` (既存)

**実装例:**
```rust
// src/core/wait.rs に追加予定
pub struct WaitArgs {
    pub selector: Option<String>,
    pub text: Option<String>, // 新規（要確認）
    pub timeout: Option<u64>,
}
```

---

### ✅ wait --timeout - タイムアウト設定

**agent-browserでの仕様:**
```bash
agent-browser wait <selector> --timeout 5000
# 5秒でタイムアウト
```

**実装状況: ✅ 完了**

**実装詳細:**
- src/core/wait.rs:196-224 にタイムアウト解析機能を実装
- 形式対応: `"10s"` (秒)、`"1m"` (分)、`"30"` (秒)
- デフォルト: 30秒
- ポーリング間隔: 500ms
- UIアイドル判定: 3回連続で要素数が変わらない

**使用例:**
```bash
agent-mobile wait visible @e1 --timeout 10s
agent-mobile wait gone @button --timeout 1m
agent-mobile wait idle --timeout 30
```

**参考実装:**
- agent-mobile: `src/core/wait.rs` (実装済み)

---

## 5. スナップショット拡張 (Snapshot)

### 🟡 snapshot -i - インタラクティブ要素のみ

**agent-browserでの仕様:**
```bash
agent-browser snapshot -i
# ボタン、リンク、入力フィールドなどのみ表示
```

**モバイル実装方針:**
- **共通**: Elementをフィルタリング（type=Button, TextField, Switch等）

**技術的検討事項:**
- 既存: snapshot コマンドあり
- 追加: `-i` / `--interactive` フラグ

**参考実装:**
- agent-mobile: `src/snapshot/mod.rs`

**実装例:**
```rust
// src/snapshot/mod.rs に追加
pub struct SnapshotArgs {
    pub interactive: bool, // 新規
    pub compact: bool,
    pub depth: Option<usize>,
    pub scope: Option<String>,
}
```

---

### 🟡 snapshot -c - 空要素削除

**agent-browserでの仕様:**
```bash
agent-browser snapshot -c
# テキストがない要素を削除
```

**モバイル実装方針:**
- **共通**: label/value/placeholderが空の要素を除外

**技術的検討事項:**
- フィルタ条件: label.is_empty() && value.is_none() && placeholder.is_none()

**参考実装:**
- agent-mobile: `src/snapshot/mod.rs`

**実装例:**
```rust
// src/snapshot/mod.rs に追加
pub struct SnapshotArgs {
    pub interactive: bool,
    pub compact: bool, // 新規
    pub depth: Option<usize>,
    pub scope: Option<String>,
}
```

---

### 🟡 snapshot -d - 階層深さ制限

**agent-browserでの仕様:**
```bash
agent-browser snapshot -d 3
# 階層深さ3まで表示
```

**モバイル実装方針:**
- **共通**: 再帰的な要素走査で深さカウント

**技術的検討事項:**
- 既存: Snapshotは階層構造を持つ
- 追加: 深さ制限ロジック

**参考実装:**
- agent-mobile: `src/snapshot/mod.rs`

**実装例:**
```rust
// src/snapshot/mod.rs に追加
pub struct SnapshotArgs {
    pub interactive: bool,
    pub compact: bool,
    pub depth: Option<usize>, // 新規
    pub scope: Option<String>,
}
```

---

### 🟡 snapshot -s - スコープ限定

**agent-browserでの仕様:**
```bash
agent-browser snapshot -s "#main"
# #main要素配下のみ表示
```

**モバイル実装方針:**
- **共通**: 指定セレクタにマッチする要素以下のみ抽出

**技術的検討事項:**
- セレクタ解決: find で要素を特定
- サブツリー抽出: 子要素のみを出力

**参考実装:**
- agent-mobile: `src/snapshot/mod.rs`

**実装例:**
```rust
// src/snapshot/mod.rs に追加
pub struct SnapshotArgs {
    pub interactive: bool,
    pub compact: bool,
    pub depth: Option<usize>,
    pub scope: Option<String>, // 新規
}
```

---

## 実装優先順位まとめ

### Phase 1（✅ 完了）
1. ✅ `is visible`, `is checked` - 既存isコマンドの拡張
   - コミット 8263dd25 で実装完了
   - is checked: value属性で "1" または "true" で判定
2. ✅ `get count`, `get box` - 既存getコマンドの拡張
   - コミット 8263dd25 で実装完了
   - get count: マッチ数をカウント
   - get box: frame情報を返す（get frameのエイリアス）
3. ✅ `wait --timeout` - 既存waitコマンドの拡張
   - 実装済み: Duration形式で "10s", "1m", "30" 対応
   - デフォルトタイムアウト: 30秒
4. 🟡 `wait --text` - テキスト出現待機
   - 実装状況: **確認が必要**
   - 計画: ポーリングで `find text` を実行

### Phase 2（未実装）
5. ❌ `check`, `uncheck` - HIDイベントで実装可能
   - UICheckbox/UISwitch のトグル操作
   - 冪等性対応：既に目的状態ならスキップ
6. ❌ `get attr` - アクセシビリティ情報から抽出
   - 計画: iOS (AX*属性) ⇔ Android (XML属性) マッピング

### Phase 3（部分実装 🟡）
7. ✅ `snapshot` フィルター - **既に大部分実装済み**
   - ✅ `-i, --interactive` - インタラクティブ要素のみ表示
   - ✅ `-c, --compact` - 空要素削除
   - ✅ `-d, --depth N` - 階層深さ制限
   - ✅ `-o, --output FILE` - ファイル出力
   - ✅ `-f, --format {text|json}` - 出力形式指定
   - ❌ `-s, --scope <selector>` - **未実装**（スコープ限定）
   - 実装箇所: src/snapshot/mod.rs lines 23-58
8. ❌ `select` - Picker操作（やや複雑）
   - iOS: UIPickerView（WheelChangedイベント）
   - Android: Spinner/DropDownMenu（adb shell input）

---

## 実装状況サマリー（更新日: 2026-01-24）

### ✅ 完了フェーズ

| コマンド | 状態 | 実装ファイル | コミット |
|---------|------|------------|---------|
| `is checked` | ✅ 完了 | src/core/is_cmd.rs:72-81 | 8263dd25 |
| `is visible` | ✅ 完了 | src/core/is_cmd.rs | 既存 |
| `is enabled` | ✅ 完了 | src/core/is_cmd.rs | 既存 |
| `get count` | ✅ 完了 | src/core/get.rs:58-93 | 8263dd25 |
| `get box` | ✅ 完了 | src/core/get.rs:111-126 | 8263dd25 |
| `get frame` | ✅ 完了 | src/core/get.rs | 既存 |
| `wait --timeout` | ✅ 完了 | src/core/wait.rs:196-224 | 既存 |
| `snapshot -i` | ✅ 完了 | src/snapshot/mod.rs:23-58 | 既存 |
| `snapshot -c` | ✅ 完了 | src/snapshot/mod.rs:23-58 | 既存 |
| `snapshot -d` | ✅ 完了 | src/snapshot/mod.rs:23-58 | 既存 |
| `snapshot -f` | ✅ 完了 | src/snapshot/mod.rs:23-58 | 既存 |

### 🟡 確認待ち

| コマンド | 状態 | 備考 |
|---------|------|------|
| `wait --text` | 🟡 要確認 | ポーリングロジック実装状況の確認が必要 |

### ❌ 未実装フェーズ

| コマンド | フェーズ | 優先度 | 技術方針 |
|---------|---------|--------|--------|
| `check` | Phase 2 | 🔴 高 | HIDイベント（tap+WheelChanged） |
| `uncheck` | Phase 2 | 🔴 高 | HIDイベント（tap） |
| `get attr` | Phase 2 | 🔴 高 | アクセシビリティ属性抽出 |
| `snapshot -s` | Phase 3 | 🟡 中 | セレクタスコープ限定 |
| `select` | Phase 3 | 🟡 中 | Picker操作（複雑） |

### コア機能実装状況（参考）

| 機能 | 状態 | 実装ファイル |
|------|------|------------|
| `tap` | ✅ | src/core/find.rs + src/platform/ios/grpc/hid.rs |
| `long-press` | ✅ | src/core |
| `fill` | ✅ | src/core |
| `type` | ✅ | src/core |
| `swipe` | ✅ | src/core |
| `scroll` | ✅ | src/core |
| `find` | ✅ | src/core/find.rs (セマンティックロケーター) |
| `screenshot` | ✅ | src/platform/ios/grpc |
| `snapshot` | ✅ | src/snapshot/mod.rs |

---

## 参考ファイルパス

- `/Users/r0227n/Dev/agent-mobile/src/core/find.rs` - セマンティックロケーター実装
- `/Users/r0227n/Dev/agent-mobile/src/core/tap.rs` - タップ実装
- `/Users/r0227n/Dev/agent-mobile/src/core/get.rs` - 情報取得実装
- `/Users/r0227n/Dev/agent-mobile/src/core/is_cmd.rs` - 状態確認実装
- `/Users/r0227n/Dev/agent-mobile/src/core/wait.rs` - 待機実装
- `/Users/r0227n/Dev/agent-mobile/src/snapshot/mod.rs` - スナップショット実装
- `/Users/r0227n/Dev/agent-mobile/src/platform/ios/grpc/hid.rs` - HIDイベント送信
- `/Users/r0227n/Dev/agent-mobile/src/platform/ios/grpc/device.rs` - アクセシビリティ情報取得
- `/Users/r0227n/Dev/agent-mobile/proto/idb.proto` - gRPC プロトコル定義
