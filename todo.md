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
- agent-mobile: `src/cli/core/find.rs` (tap実装)
- agent-mobile: `src/platform/ios/grpc/hid.rs` (HIDイベント送信)

**実装例:**
```rust
// src/cli/core/select.rs (新規)
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
- agent-mobile: `src/cli/core/tap.rs` (タップ実装)
- agent-mobile: `src/cli/snapshot/types.rs` (Element.enabled)

**実装例:**
```rust
// src/cli/core/check.rs (新規)
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
- agent-mobile: `src/cli/core/get.rs` (既存)
- agent-mobile: `src/cli/snapshot/extractor/extractor_ios.rs` (属性抽出)

**実装例:**
```rust
// src/cli/core/get.rs に追加
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
- agent-mobile: `src/cli/core/find.rs` (--all実装済み)

**実装例:**
```rust
// src/cli/core/get.rs に追加
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
- agent-mobile: `src/cli/snapshot/types.rs` (Element.frame)

**実装例:**
```rust
// src/cli/core/get.rs に追加
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
- agent-mobile: `src/cli/core/is_cmd.rs` (既存)

**実装例:**
```rust
// src/cli/core/is_cmd.rs に追加
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
- agent-mobile: `src/cli/core/is_cmd.rs`

**実装例:**
```rust
// src/cli/core/is_cmd.rs に追加
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

**参考実装:**
- agent-mobile: `src/cli/core/wait.rs` (既存)

**実装例:**
```rust
// src/cli/core/wait.rs に追加
pub struct WaitArgs {
    pub selector: Option<String>,
    pub text: Option<String>, // 新規
    pub timeout: Option<u64>,
}
```

---

### 🔴 wait --timeout - タイムアウト設定

**agent-browserでの仕様:**
```bash
agent-browser wait <selector> --timeout 5000
# 5秒でタイムアウト
```

**モバイル実装方針:**
- **共通**: 既存のwaitコマンドに `--timeout` オプション追加

**技術的検討事項:**
- デフォルト: 30000ms
- カスタマイズ可能に

**参考実装:**
- agent-mobile: `src/cli/core/wait.rs`

**実装例:**
```rust
// src/cli/core/wait.rs に追加（既にtimeoutフィールドがあるか確認）
```

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
- agent-mobile: `src/cli/snapshot/mod.rs`

**実装例:**
```rust
// src/cli/snapshot/mod.rs に追加
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
- agent-mobile: `src/cli/snapshot/mod.rs`

**実装例:**
```rust
// src/cli/snapshot/mod.rs に追加
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
- agent-mobile: `src/cli/snapshot/mod.rs`

**実装例:**
```rust
// src/cli/snapshot/mod.rs に追加
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
- agent-mobile: `src/cli/snapshot/mod.rs`

**実装例:**
```rust
// src/cli/snapshot/mod.rs に追加
pub struct SnapshotArgs {
    pub interactive: bool,
    pub compact: bool,
    pub depth: Option<usize>,
    pub scope: Option<String>, // 新規
}
```

---

## 実装優先順位まとめ

### Phase 1（即座に実装）
1. `is visible`, `is checked` - 既存isコマンドの拡張
2. `get count`, `get box` - 既存getコマンドの拡張
3. `wait --text`, `wait --timeout` - 既存waitコマンドの拡張

### Phase 2（短期）
4. `check`, `uncheck` - HIDイベントで実装可能
5. `get attr` - アクセシビリティ情報から抽出

### Phase 3（中期）
6. `select` - Picker操作（やや複雑）
7. `snapshot` フィルター - 既存機能の拡張

---

## 参考ファイルパス

- `/Users/r0227n/Dev/agent-mobile/src/cli/core/find.rs` - セマンティックロケーター実装
- `/Users/r0227n/Dev/agent-mobile/src/cli/core/tap.rs` - タップ実装
- `/Users/r0227n/Dev/agent-mobile/src/cli/core/get.rs` - 情報取得実装
- `/Users/r0227n/Dev/agent-mobile/src/cli/core/is_cmd.rs` - 状態確認実装
- `/Users/r0227n/Dev/agent-mobile/src/cli/core/wait.rs` - 待機実装
- `/Users/r0227n/Dev/agent-mobile/src/cli/snapshot/mod.rs` - スナップショット実装
- `/Users/r0227n/Dev/agent-mobile/src/platform/ios/grpc/hid.rs` - HIDイベント送信
- `/Users/r0227n/Dev/agent-mobile/src/platform/ios/grpc/device.rs` - アクセシビリティ情報取得
- `/Users/r0227n/Dev/agent-mobile/proto/idb.proto` - gRPC プロトコル定義
