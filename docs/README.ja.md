# agent-mobile 日本語リファレンス

> このドキュメントは [../README.md](../README.md) の日本語版です。

[English](../README.md)

## 追加ドキュメント

- [iOS Runner ドキュメント](ios-runner.ja.md)

`agent-mobile` は、AI エージェントによるモバイルアプリのテストと自動化を想定した Rust 製 CLI です。現在の UI を取得し、`@e1` のような短い要素参照を生成し、`tap`、`fill`、`find`、`wait`、`screenshot` といった簡潔なコマンドでアプリを操作できます。

> [!NOTE]
> このプロジェクトは [agent-browser](https://github.com/vercel-labs/agent-browser) に着想を得ています。

> [!IMPORTANT]
> `agent-mobile` は現在も鋭意開発中です。今後の開発に伴い、コマンド、挙動、API は変更される可能性があります。

## なぜ agent-mobile なのか

- **LLM ループ向けにコンパクト**: 長いセレクタより `tap @e3` のほうが短く、扱いやすいです。
- **セマンティックロケーター対応**: `text`、`label`、`placeholder`、`type`、状態で UI 要素を見つけられます。
- **クロスプラットフォーム**: iOS と Android を単一の CLI で扱えます。
- **セッション管理**: デバイスを名前付きセッションに紐づけて再利用できます。
- **機械可読な出力**: 参照系コマンドは JSON 出力に対応しています。

## できること

- UI ツリーを取得し、要素参照を生成する
- タップ、ロングプレス、入力、フィル、スクロール、スワイプ
- アプリの起動、インストール、終了、アンインストール
- 要素やテキストの出現、消失を待つ
- スクリーンショット保存、動画録画、コンソールログの取得
- デバイスと名前付きセッションの管理

## インストール

### 前提条件

- Rust ツールチェーン
- iOS シミュレータ対応のための macOS + Xcode Command Line Tools
- Android 対応のための Android SDK Platform Tools (`adb`)

### ソースからビルド

```bash
git clone https://github.com/r0227n/agent-mobile.git
cd agent-mobile

cargo build --release
cargo install --path .

agent-mobile doctor
agent-mobile --help
```

### Cargo の bin を PATH に追加

`cargo install` 後に `agent-mobile` が見つからない場合は、Cargo の bin ディレクトリを `PATH` に追加してください。

まず、Cargo がバイナリをどこにインストールしたか確認します。

```bash
cargo install --path .
cargo install --list | rg '^agent-mobile '
```

多くの環境では、次のいずれかが使われます。

- `$CARGO_HOME/bin` (`CARGO_HOME` を設定している場合)
- `$HOME/.cargo/bin` (デフォルト)

現在のシェルだけで有効にする場合:

```bash
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
agent-mobile --help
```

永続化する場合は、使っているシェルの設定ファイルに追記してください。

- zsh: `~/.zshrc`
- bash: `~/.bashrc` または `~/.bash_profile`
- fish: `fish_add_path (string join / (or $CARGO_HOME $HOME/.cargo) bin)` を実行

その後、新しいターミナルを開くか設定を再読み込みし、次で確認します。

```bash
command -v agent-mobile
agent-mobile --help
```

### プラットフォーム別メモ

**iOS**

```bash
xcode-select --install
```

`agent-mobile` は `xcrun simctl` と同梱の XCUITest Runner を使って iOS シミュレータを操作します。

**Android**

Android Studio または Android SDK Platform Tools を導入し、`adb` が `PATH` に通っていることを確認してください。

```bash
export ANDROID_HOME="$HOME/Library/Android/sdk"
export PATH="$PATH:$ANDROID_HOME/platform-tools"
adb version
```

## クイックスタート

### 1. 環境を確認する

```bash
agent-mobile doctor
```

### 2. デバイスを選ぶ

```bash
agent-mobile device list
agent-mobile device boot "iPhone 15 Pro"
```

### 3. アプリを起動する

```bash
agent-mobile app launch com.apple.mobilesafari
```

### 4. 現在の UI を取得する

```bash
agent-mobile snapshot
```

出力例:

```text
@e1  Button     "Continue"                     [enabled]
@e2  TextField  "Search or enter website name" [enabled]
@e3  Button     "Cancel"                       [enabled]
```

### 5. UI を操作する

```bash
agent-mobile tap @e1
agent-mobile fill @e2 "https://example.com"
```

セマンティックロケーターを直接使うこともできます。

```bash
agent-mobile find text "Continue" tap
agent-mobile find placeholder "Search or enter website name" fill "https://example.com"
```

### 6. 結果を確認する

```bash
agent-mobile wait text "Example Domain" --timeout 10s
agent-mobile screenshot --output result.png
```

## 主要コンセプト

### 1. Snapshot と要素参照

`snapshot` はアクセシビリティツリーを取得し、`@e1`、`@e2`、`@e3` のような短い参照を割り当てます。

```bash
agent-mobile snapshot
agent-mobile tap @e1
agent-mobile get @e2 text
```

最短のやり取りで操作したい場合は、この要素参照が有効です。

### 2. セマンティックロケーター

生成済みの参照を使わず、意味ベースで要素を検索することもできます。

```bash
agent-mobile find text "Login"
agent-mobile find label "Email" fill "user@example.com"
agent-mobile find type Button --nth 0 tap
agent-mobile find enabled --all -f json
```

対応しているロケータ種別:

- `type`
- `text`
- `label`
- `placeholder`
- `enabled`
- `disabled`

インラインアクション:

- `tap`
- `long-press`
- `fill`
- `clear`

### 3. セッション

セッションを使うと、デバイス UDID を名前に紐づけてコマンド間で再利用できます。

```bash
agent-mobile session create ios-dev --udid <UDID>
agent-mobile --session ios-dev snapshot

export AGENT_MOBILE_SESSION=ios-dev
agent-mobile session show
```

### 4. JSON 出力

参照系コマンドは、`-f json` のようなコマンドごとのフラグで機械可読な出力を返せます。

```bash
agent-mobile snapshot -f json
agent-mobile device list -f json
agent-mobile session list -f json
agent-mobile doctor --json
```

## コマンド一覧

| 分類 | コマンド |
| --- | --- |
| 操作 | `tap`, `long-press`, `fill`, `type`, `check`, `uncheck`, `select`, `scroll`, `swipe` |
| UI 参照 | `snapshot`, `find`, `get`, `is`, `wait` |
| 画像とログ | `screenshot`, `record`, `console` |
| アプリ管理 | `app launch`, `app terminate`, `app install`, `app uninstall`, `app list`, `app grant`, `app revoke`, `app reset` |
| デバイス管理 | `device list`, `device boot`, `device shutdown`, `device pbcopy`, `device pbpaste` |
| セッション管理 | `session create`, `session list`, `session show`, `session destroy` |
| 環境診断 | `doctor` |

詳細は `agent-mobile <command> --help` を参照してください。

## 例: ログインフロー

```bash
agent-mobile app launch com.example.app
agent-mobile snapshot

agent-mobile fill @e1 "user@example.com"
agent-mobile fill @e2 "password123"
agent-mobile tap @e3

agent-mobile wait text "Profile" --timeout 5s
agent-mobile screenshot --output logged_in.png
```

## アーキテクチャ概要

- `agent-mobile`: Rust 製 CLI エントリポイント
- `crates/core`: 共通型とトレイト
- `crates/platform-ios`: `simctl` と XCUITest Runner を使った iOS 実装
- `crates/platform-android`: `adb` と UI Automator を使った Android 実装
- `crates/gateway`: 上位のプラットフォーム解決とオーケストレーション
- `crates/xcuitest-runner`: iOS 自動化で使う Swift 側 HTTP サーバー

## 関連ドキュメント

- [../README.md](../README.md) - 英語版 README
- [README.md](./README.md) - docs のインデックス
- [../CLAUDE.md](../CLAUDE.md) - 開発ワークフロー
- [../AGENTS.md](../AGENTS.md) - エージェント向けリポジトリガイド

## コントリビューション

CLI の挙動を変える場合は、ドキュメント更新に加えて、実機またはシミュレータでの動作確認も行ってください。

## ライセンス

MIT。詳細は [../LICENSE](../LICENSE) を参照してください。
