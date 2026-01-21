# agent-mobile Specification

**AI Agent 向け モバイルアプリ E2E テスト CLI**

## 概要

agent-mobile は iOS/Android シミュレータ・実機上でモバイルアプリの E2E テストを実行するための CLI ツールです。アクセシビリティツリーに基づく **ref 識別子システム** により、AI Agent が効率的に UI 操作を実行できるよう設計されています。

## 設計哲学

1. **アクセシビリティ優先**: DOM や View 階層ではなく、アクセシビリティツリーを操作の基盤とする
2. **ref 識別子**: 各要素に `@e1`, `@e2` のような安定した参照を付与し、セレクタの脆弱性を排除
3. **AI 最適化**: LLM のコンテキスト効率を考慮した構造化出力
4. **セッション分離**: 複数のシミュレータ/デバイスを並行操作可能

---

## Core Commands

AI Agent が主に使用するコマンドセット。シンプルで直感的な操作を提供。

### snapshot - アクセシビリティツリー取得

```bash
agent-mobile snapshot [options]
```

アクセシビリティツリーを ref 付きで出力。AI Agent の状態認識に使用。

**オプション:**
| オプション | 説明 |
|-----------|------|
| `-i, --interactive` | 操作可能な要素のみ表示 |
| `-c, --compact` | 空の構造要素を除去 |
| `-d, --depth <n>` | ツリー深さ制限 |
| `-o, --output <path>` | 結果をファイルに保存 |
| `--format <json\|text>` | 出力形式 (デフォルト: text) |

**出力例:**
```
Screen "Home"
├── Button "Login" [ref=@e1]
├── TextField "Email" [ref=@e2] placeholder="Enter email"
├── TextField "Password" [ref=@e3] placeholder="Enter password"
├── Button "Forgot Password?" [ref=@e4]
└── StaticText "Don't have an account?"
    └── Link "Sign up" [ref=@e5]
```

**JSON 出力例:**
```json
{
  "snapshot_id": "snap_abc123",
  "timestamp": "2024-01-21T10:30:00Z",
  "elements": [
    {
      "ref": "@e1",
      "type": "Button",
      "label": "Login",
      "frame": {"x": 100, "y": 400, "width": 200, "height": 44},
      "enabled": true,
      "traits": ["button"]
    }
  ]
}
```

**オプションロジック:**

#### `-i, --interactive` フィルタリング

操作可能な要素のみを出力する。以下の traits/type を持つ要素を抽出：

| 対象要素 | iOS traits | 備考 |
|---------|-----------|------|
| Button | `.button` | タップ可能 |
| TextField | `.searchField`, `.textField` | テキスト入力可能 |
| SecureTextField | `.secureTextField` | パスワード入力 |
| Switch | `.toggleButton` | ON/OFF 切替 |
| Slider | `.adjustable` | 値調整可能 |
| Link | `.link` | ナビゲーション |
| Cell | `.cell` + `.selectable` | 選択可能なセル |
| Tab | `.tabBar` 内の要素 | タブ切替 |

**処理フロー:**
```
1. アクセシビリティツリー全体を取得
2. 各要素の traits をチェック
3. 操作可能な traits を持つ要素のみ残す
4. 親要素は子に操作可能要素があれば残す（構造保持）
5. ref を操作可能要素にのみ付与
```

#### `-c, --compact` 圧縮

空の構造要素（子がない Container, View 等）を除去：

```
適用前:
Screen "Home"
├── View
│   └── View
│       └── Button "Login" [ref=@e1]
└── View (空)

適用後 (-c):
Screen "Home"
└── Button "Login" [ref=@e1]
```

**除去対象:**
- 子要素がない View, Container, Group
- label/value が空の StaticText
- 視覚的に非表示 (frame が 0x0) の要素

#### `-d, --depth <n>` 深さ制限

ツリーの探索深度を制限：

```
-d 2 の場合:
Screen "Home"          (depth=0)
├── NavigationBar      (depth=1)
│   └── Button "Back"  (depth=2) ← ここまで
│       └── Icon       (depth=3) ← 省略
└── ScrollView         (depth=1)
    └── Cell "Item"    (depth=2) ← ここまで
```

**注意:** depth 制限された場合、制限境界に `...` または `(n more)` を表示

#### `-o, --output <path>` ファイル出力

snapshot 結果をファイルに保存：

```bash
agent-mobile snapshot -o ./snapshot.txt
agent-mobile snapshot --format json -o ./snapshot.json
```

**動作:**
- 指定パスにスナップショット結果を書き出し
- stdout には何も出力しない（エラー時のみ stderr）
- 既存ファイルは上書き
- ディレクトリが存在しない場合はエラー

**ユースケース:**
- AI Agent が後続コマンドでファイルを参照（`--snapshot` オプションと組み合わせ）
- デバッグ用にスナップショットを保存
- 複数セッション間でスナップショットを共有

#### `--format <json|text>` 出力形式

| 形式 | 用途 | 特徴 |
|-----|------|------|
| `text` (デフォルト) | AI Agent の視覚的理解 | 人間可読、ツリー構造 |
| `json` | プログラマティック処理 | 座標情報完全、パース可能 |

#### オプション組み合わせ

```bash
# 推奨: AI Agent 用
agent-mobile snapshot -i -c

# デバッグ用: 全要素表示
agent-mobile snapshot --format json

# 大規模画面: 深さ制限付き
agent-mobile snapshot -i -c -d 3

# ファイル出力（後続コマンドで参照）
agent-mobile snapshot -i -c --format json -o ./current.json
agent-mobile tap @e1 --snapshot ./current.json
```

### tap - 要素タップ

```bash
agent-mobile tap <ref|text>
agent-mobile tap @e1          # ref でタップ
agent-mobile tap "Login"     # テキストでタップ
```

### fill - テキスト入力

```bash
agent-mobile fill <ref|text> <value>
agent-mobile fill @e2 "user@example.com"
agent-mobile fill "Email" "user@example.com"
```

フィールドをクリア後、テキストを入力。

### type - キー入力

```bash
agent-mobile type <text>
agent-mobile type "Hello World"
```

現在フォーカスされている要素にテキストを入力（クリアせず追記）。

### swipe - スワイプ

```bash
agent-mobile swipe <direction> [--from <ref>]
agent-mobile swipe up
agent-mobile swipe down --from @e3
agent-mobile swipe left --distance 300
```

**方向:** `up`, `down`, `left`, `right`

### scroll - スクロール

```bash
agent-mobile scroll <direction> [--in <ref>]
agent-mobile scroll down
agent-mobile scroll down --in @e5    # 特定のスクロールビュー内
```

### tap - ボタン/キー押下

```bash
agent-mobile tap <key>
agent-mobile tap home
agent-mobile tap back
agent-mobile tap enter
```

**キー:** `home`, `back`, `enter`, `tab`, `escape`, `delete`

### get - 要素情報取得

```bash
agent-mobile get text @e1     # テキスト取得
agent-mobile get value @e2    # 値取得 (入力フィールド)
agent-mobile get attr @e1 enabled  # 属性取得
```

### is - 状態確認

```bash
agent-mobile is visible @e1   # 表示されているか
agent-mobile is enabled @e1   # 有効か
agent-mobile is focused @e2   # フォーカスされているか
```

### wait - 待機

```bash
agent-mobile wait visible <ref|text> [--timeout 30s]
agent-mobile wait gone <ref|text>
agent-mobile wait idle              # アニメーション完了待ち
```

### screenshot - スクリーンショット

```bash
agent-mobile screenshot [path]
agent-mobile screenshot                    # stdout にbase64出力
agent-mobile screenshot ./screen.png
```

---

## App Commands

アプリケーションライフサイクル管理。

```bash
agent-mobile app launch <bundle-id>
agent-mobile app terminate <bundle-id>
agent-mobile app install <path>
agent-mobile app uninstall <bundle-id>
agent-mobile app list
agent-mobile app grant <permission> --bundle <bundle-id>
agent-mobile app revoke <permission> --bundle <bundle-id>
```

---

## Device Commands

デバイス/シミュレータ管理。

```bash
agent-mobile device list
agent-mobile device boot <name|udid>
agent-mobile device shutdown <udid>
agent-mobile device pbcopy <text>     # クリップボードにコピー
agent-mobile device pbpaste           # クリップボードから取得
```

---

## Session Management

複数のデバイス/シミュレータを並行操作。

### セッション指定

```bash
# コマンドラインオプション
agent-mobile --session sim1 snapshot
agent-mobile --session sim2 tap @e1

# 環境変数
AGENT_MOBILE_SESSION=sim1 agent-mobile snapshot
```

### セッションコマンド

```bash
agent-mobile session list              # アクティブセッション一覧
agent-mobile session show              # 現在のセッション情報
agent-mobile session create <name> --udid <udid>
agent-mobile session destroy <name>
```

### セッション状態

各セッションは以下を保持:
- 対象デバイス UDID
- 最新の snapshot とその ref マッピング
- アプリ状態

**セッション一覧出力例:**
```
Sessions:
  sim1 (active)
    UDID: ABC123-DEF456
    Device: iPhone 15 Pro (iOS 17.2)
    App: com.example.app
    Last snapshot: 2024-01-21T10:30:00Z (15 refs)

  sim2
    UDID: GHI789-JKL012
    Device: iPhone 15 (iOS 17.2)
    App: com.example.app
    Last snapshot: 2024-01-21T10:28:00Z (12 refs)
```

---

## Ref System 詳細

### Ref の生成ルール

1. `snapshot` コマンド実行時に全要素に ref を割り当て
2. ref は `@e1`, `@e2`, ... の連番形式 (agent-browser 互換)
3. 操作可能な要素（Button, TextField, Link 等）を優先的に番号付け
4. 同一セッション内で snapshot を再取得すると ref は再割り当て

### Ref のライフサイクル

```
snapshot実行 → ref生成 → tap/fill等で使用 → 画面遷移 → snapshot再取得 → 新ref生成
```

### テキストフォールバック

ref が不明な場合はテキストで指定可能:

```bash
agent-mobile tap "Submit"     # "Submit" というラベルを持つ要素をタップ
agent-mobile tap @e3           # ref @e3 をタップ (推奨)
```

---

### Core Commands の Ref 解決ロジック

Core Commands (`tap`, `fill`, `swipe --from`, `scroll --in`, `get`, `is`, `wait`) が ref を座標/要素に変換する仕組み。

#### 解決フロー

```
┌─────────────────────────────────────────────────────────────┐
│ 1. 引数パース                                               │
│    tap @e1  →  identifier = "@e1"                          │
│    tap "Login" → identifier = "Login"                      │
│    tap @e1 --snapshot ./file.json → snapshot_file 設定     │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. スナップショットソース決定                                │
│    --snapshot 指定あり → ファイルを読み込み                 │
│    --snapshot 指定なし → セッションの最新スナップショット    │
│    いずれもなし → NO_SNAPSHOT エラー                        │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. 識別子タイプ判定                                         │
│    @e で始まる → RefIdentifier                              │
│    それ以外   → TextIdentifier                              │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 4a. RefIdentifier の場合                                    │
│     - スナップショットの ref マッピングを参照               │
│     - @e1 → {frame: {x, y, w, h}, ...} を取得               │
│     - マッピングになければ ELEMENT_NOT_FOUND エラー          │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 4b. TextIdentifier の場合                                   │
│     - 現在の画面でアクセシビリティ検索を実行                │
│     - label/value が一致する要素を検索                      │
│     - 複数ヒット時は最初の操作可能要素を選択                │
│     - 見つからなければ ELEMENT_NOT_FOUND エラー              │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. 座標計算                                                 │
│    frame.center_x = frame.x + frame.width / 2              │
│    frame.center_y = frame.y + frame.height / 2             │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│ 6. 低レベルコマンド実行                                     │
│    tap @e1 → hid tap <center_x> <center_y>                 │
│    fill @e2 "text" → hid tap → hid clear → hid text        │
└─────────────────────────────────────────────────────────────┘
```

#### Ref マッピング構造

セッション内で保持される ref マッピング：

```rust
struct RefMapping {
    snapshot_id: String,           // "snap_abc123"
    timestamp: DateTime<Utc>,
    refs: HashMap<String, ElementRef>,  // "@e1" -> ElementRef
}

struct ElementRef {
    ref_id: String,                // "@e1"
    element_type: String,          // "Button"
    label: Option<String>,         // "Login"
    value: Option<String>,
    frame: Frame,                  // {x, y, width, height}
    enabled: bool,
    traits: Vec<String>,
}
```

#### コマンド別の解決動作

| コマンド | 解決後の動作 |
|---------|-------------|
| `tap @e1` | frame 中心座標で `hid tap` |
| `fill @e2 "text"` | frame 中心で tap → `hid clear` → `hid text "text"` |
| `swipe up --from @e3` | @e3 の frame 中心を起点にスワイプ |
| `scroll down --in @e4` | @e4 の ScrollView 内でスクロール |
| `get text @e1` | @e1 の label/value を返す |
| `is enabled @e1` | @e1 の enabled 属性を返す |
| `wait visible @e1` | ポーリングで @e1 が表示されるまで待機 |

#### エラーハンドリング

```json
// ref が見つからない場合
{
  "success": false,
  "error": {
    "code": "ELEMENT_NOT_FOUND",
    "message": "Element with ref @e5 not found in current snapshot",
    "details": {
      "ref": "@e5",
      "snapshot_id": "snap_abc123",
      "hint": "Run 'snapshot' to refresh element references"
    }
  }
}

// スナップショットがない場合
{
  "success": false,
  "error": {
    "code": "NO_SNAPSHOT",
    "message": "No snapshot available. Run 'snapshot' first or specify --snapshot <file>",
    "details": {
      "session": "sim1",
      "hint": "Run 'agent-mobile snapshot -o ./snap.json' to create a snapshot"
    }
  }
}

// --snapshot で指定したファイルが見つからない場合
{
  "success": false,
  "error": {
    "code": "SNAPSHOT_FILE_NOT_FOUND",
    "message": "Snapshot file not found: ./missing.json",
    "details": {
      "path": "./missing.json"
    }
  }
}

// snapshot が古い場合の警告 (操作は実行)
{
  "success": true,
  "data": { ... },
  "warning": {
    "code": "STALE_SNAPSHOT",
    "message": "Snapshot is older than 30 seconds",
    "snapshot_age_seconds": 45
  }
}
```

#### RefIdentifier vs TextIdentifier 比較

| 観点 | RefIdentifier (`@e1`) | TextIdentifier (`"Login"`) |
|------|----------------------|---------------------------|
| 解決速度 | 高速 (HashMap lookup) | 低速 (画面検索が必要) |
| 精度 | 100% (スナップショット時点で確定) | 曖昧性あり (同名要素) |
| 画面変化耐性 | 低 (snapshot 再取得必要) | 高 (動的検索) |
| 推奨用途 | 通常操作 | ref 不明時のフォールバック |

---

## AI Agent ワークフロー例

### 基本的なログインフロー

```bash
# 1. 現在の画面状態を取得
$ agent-mobile snapshot -i

Screen "Login"
├── TextField "Email" [ref=@e1]
├── TextField "Password" [ref=@e2]
└── Button "Sign In" [ref=@e3]

# 2. メールアドレス入力
$ agent-mobile fill @e1 "test@example.com"

# 3. パスワード入力
$ agent-mobile fill @e2 "password123"

# 4. ログインボタンタップ
$ agent-mobile tap @e3

# 5. 画面遷移後、新しい状態を取得
$ agent-mobile wait idle
$ agent-mobile snapshot -i

Screen "Home"
├── Button "Profile" [ref=@e1]
├── Button "Settings" [ref=@e2]
└── ScrollView [ref=@e3]
    ├── Cell "Item 1" [ref=@e4]
    └── Cell "Item 2" [ref=@e5]
```

### 並行テスト実行

```bash
# セッション1: iPhone 15 Pro
$ agent-mobile session create iphone15pro --udid ABC123
$ agent-mobile --session iphone15pro app launch com.example.app
$ agent-mobile --session iphone15pro snapshot

# セッション2: iPhone SE
$ agent-mobile session create iphonese --udid DEF456
$ agent-mobile --session iphonese app launch com.example.app
$ agent-mobile --session iphonese snapshot

# 同時に操作
$ agent-mobile --session iphone15pro tap @e1
$ agent-mobile --session iphonese tap @e1
```

---

## 出力形式

### 標準出力規約

- **成功時**: 結果データを stdout に出力
- **エラー時**: エラーメッセージを stderr に出力、exit code 非ゼロ
- **--format json**: 全コマンドで JSON 出力をサポート

### JSON 形式の統一

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

エラー時:
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "ELEMENT_NOT_FOUND",
    "message": "Element with ref @e5 not found",
    "details": { "ref": "@e5" }
  }
}
```

---

## グローバルオプション

```
--udid <udid>       対象デバイスを指定
--session <name>    セッションを指定
--timeout <sec>     タイムアウト秒数 (デフォルト: 30)
--format <json|text>  出力形式 (デフォルト: text)
--verbose           詳細ログ出力
--help              ヘルプ表示
```

### Core Commands 共通オプション

ref を使用するコマンド（`tap`, `fill`, `swipe --from`, `scroll --in`, `get`, `is`, `wait`）で利用可能：

| オプション | 説明 |
|-----------|------|
| `--snapshot <path>` | ref 解決に使用するスナップショットファイルを指定 |

#### `--snapshot <path>` の動作

```bash
# 通常: セッション内の最新スナップショットを参照
agent-mobile tap @e1

# ファイル指定: 指定ファイルから ref を解決
agent-mobile tap @e1 --snapshot ./snapshot.json
```

**解決優先順位:**
1. `--snapshot` で指定されたファイル
2. セッション内の最新スナップショット
3. いずれもない場合は `NO_SNAPSHOT` エラー

**ファイル形式:**
- `snapshot --format json` で出力された JSON ファイルのみ対応
- text 形式は `--snapshot` での参照不可

**ユースケース:**

```bash
# 1. スナップショットをファイルに保存
agent-mobile snapshot -i --format json -o ./state.json

# 2. 複数コマンドで同じスナップショットを参照（画面変化があっても一貫した ref）
agent-mobile tap @e1 --snapshot ./state.json
agent-mobile fill @e2 "text" --snapshot ./state.json
agent-mobile tap @e3 --snapshot ./state.json

# 3. 別セッションのスナップショットを参照
agent-mobile --session sim1 snapshot -i --format json -o ./sim1.json
agent-mobile --session sim2 tap @e1 --snapshot ./sim1.json  # sim1 の ref を sim2 で使用（非推奨だが可能）
```

**注意:**
- `--snapshot` 使用時もセッションの UDID に対して操作が実行される
- ファイル内の frame 座標が現在の画面と一致しない場合、意図しない位置がタップされる可能性あり
- 画面遷移後は新しい snapshot を取得することを推奨

---

## エラーコード

| コード | 説明 |
|-------|------|
| `ELEMENT_NOT_FOUND` | 指定された ref/テキストの要素が見つからない |
| `SESSION_NOT_FOUND` | 指定されたセッションが存在しない |
| `DEVICE_NOT_FOUND` | 指定されたデバイスが見つからない |
| `APP_NOT_RUNNING` | アプリが起動していない |
| `TIMEOUT` | 操作がタイムアウト |
| `PERMISSION_DENIED` | 権限不足 |
| `NO_SNAPSHOT` | スナップショットが存在しない（ref 解決不可） |
| `SNAPSHOT_FILE_NOT_FOUND` | 指定されたスナップショットファイルが見つからない |

---

## 今後の拡張候補

1. **find コマンド**: セマンティック検索 (`find role button`, `find text "Submit"`)
2. **assert コマンド**: テストアサーション (`assert visible @e1`)
3. **record/replay**: 操作記録と再生
4. **network コマンド**: ネットワークモック/インターセプト
5. **Android サポート強化**: 同一 API での Android 操作

---

## 既存コマンドとの互換性

現在の `hid`, `app`, `device`, `idb` コマンドは引き続き利用可能。
Core Commands はこれらの上位抽象レイヤーとして機能。

### コマンド階層

```
Core Commands (AI Agent 向け高レベル API)
│
├── snapshot    → 新規実装 (UI要素収集 + ref 付与)
├── tap @e1     → ref解決 → hid tap <x> <y>
├── fill @e2    → ref解決 → hid tap <x> <y> + hid clear + hid text
├── swipe       → hid swipe
├── scroll      → hid scroll
├── tap       → hid button / hid key
├── type        → hid text
├── get/is      → snapshot + 情報抽出
├── wait        → ポーリング実装
└── screenshot  → idb screenshot

既存コマンド (詳細制御用)
│
├── hid         → タッチ、キーボード、ボタン操作
├── app         → アプリライフサイクル
├── device      → デバイス管理
└── idb         → iOS 高度機能 (59 サブコマンド)
        ↓
Platform Layer (iOS gRPC / Android ADB)
```

### 使い分け指針

| ユースケース | 推奨コマンド |
|-------------|-------------|
| AI Agent による E2E テスト | Core Commands (`tap @e1`, `fill @e2`) |
| 座標ベースの精密操作 | `hid tap 100 200` |
| デバッグ・調査 | `snapshot`, `idb accessibility` |
| iOS 固有機能 | `idb accessibility`, `idb xctest-run` |

---

## 検証方法

### Core Commands の動作確認

```bash
# 1. セッション作成
agent-mobile session create test --udid <simulator-udid>

# 2. アプリ起動
agent-mobile app launch com.example.app

# 3. スナップショット取得
agent-mobile snapshot -i

# 4. ref を使った操作
agent-mobile tap @e1
agent-mobile fill @e2 "test@example.com"

# 5. 再度スナップショットで状態確認
agent-mobile snapshot -i
```

### テスト項目

- [ ] snapshot で ref が正しく付与される
- [ ] tap @e1 で正しい座標がタップされる
- [ ] fill でテキストクリア → 入力が動作する
- [ ] session 切り替えで ref マッピングが分離される
- [ ] エラー時に適切なエラーコードが返る
