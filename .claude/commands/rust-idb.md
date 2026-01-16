---
name: rust-idb
allowed-tools: Bash(cargo test:*), Bash(./target/debug/agent-mobile:*), AskUserQuestion
description: Python idbコマンドをRust実装に自動移植します。
context: fork
agent: cli-developer, python-engineer, rust-engineer
---

# Rust IDB Command Porter

このスキルは、Python版idbコマンドをRust実装（agent-mobile）に自動移植します。

## 引数

`$ARGUMENTS` から以下を抽出してください：
- `command_name`: 移植するコマンド名（例: screenshot, kill, install）
- `--skip-tests`: オプションフラグ、テスト作成をスキップ

## ワークフロー

### Phase 1: 検証と解析

**Step 1.1: 引数を解析**
- `$ARGUMENTS` からコマンド名と `--skip-tests` フラグを抽出
- コマンド名がない場合はエラーを表示して終了

**Step 1.2: 既存実装をチェック**
- `src/cli/idb/mod.rs` を読み取り、enum `IdbCommands` 内を検索
- コマンドが既に存在する場合:
  ```
  コマンド '<command>' は既に実装されています。

  実装ファイル:
  - src/cli/idb/<command>.rs
  - src/cli/idb/mod.rs:<line>
  - tests/<command>_integration.rs

  レビューまたは拡張を希望しますか？
  ```
  ユーザーの回答を待って、"no" の場合は終了

**Step 1.3: Python実装を検索**
- `idb/idb/cli/commands/<command>.py` を読み取り
- ファイルが存在しない場合:
  - `idb/idb/cli/commands/` 内の全ファイルをリスト
  - 類似するコマンド名を提案（例: launsh → launch）
  - ユーザーに確認を求める

**Step 1.4: コマンドタイプを識別**
Pythonコードから以下を判定:
- `BaseCommand` を継承しているか
- `ClientCommand` → 特定デバイスへの操作、`run_with_client()` メソッド
- `ManagementCommand` → 複数デバイス管理、`run_with_manager()` メソッド
- `CompanionCommand` → companion daemon操作、`run_with_companion()` メソッド

### Phase 2: 実装情報の収集

**Step 2.1: Pythonコードを解析**
以下の情報を抽出:
1. `@property def description(self) -> str:` からコマンド説明
2. `@property def name(self) -> str:` からコマンド名
3. `def add_parser_arguments(self, parser: ArgumentParser):` から引数定義:
   - 位置引数: `parser.add_argument("arg_name", ...)`
   - オプション引数: `parser.add_argument("--flag", ...)`
   - フラグ: `action="store_true"`
   - デフォルト値: `default=...`
   - 型: `type=str/int/etc`
4. `async def run_with_*` メソッド内から:
   - 呼び出される `client.*()` または `manager.*()` メソッド
   - 引数の渡し方
   - 戻り値の処理方法
   - 出力フォーマット（JSON, テキスト, バイナリ）

**Step 2.2: gRPCメソッドを特定**
- Python client/manager メソッドから、必要なgRPCメソッドを識別
- 例: `client.screenshot()` → `rpc screenshot(...)`
- `idb/idb/grpc/client.py` を参照して、実際のgRPC呼び出しを確認

**Step 2.3: Protoメッセージを特定**
- `idb/proto/idb.proto` を検索
- 必要なメッセージ定義（Request, Response）を特定
- 現在の `proto/idb.proto` に存在するか確認

**Step 2.4: 分析結果をユーザーに提示**

Phase 2 完了時点で、収集した情報を整理してユーザーに提示:

```
## コマンド分析完了: <command-name>

### 基本情報:
- コマンド名: <name>
- 説明: <description>
- タイプ: [ClientCommand/ManagementCommand/CompanionCommand]

### 必要な実装:
- 引数: <list of arguments>
- gRPCメソッド: <list of gRPC calls>
- protoメッセージ: <list of proto messages to add/already exist>
- 特殊処理: <any special handling needed (streaming, binary output, signal handling, etc.)>

### 変更予定ファイル:
- proto/idb.proto (X 個のメッセージ追加 / 変更なし)
- src/cli/idb/mod.rs (enum バリアント追加)
- src/cli/idb/<command>.rs (新規作成、約 XX 行)
- src/grpc/client.rs (X 個のメソッド追加)
- src/main.rs (ルーティング追加)
- tests/<command>_integration.rs (新規作成、X 個のテスト)

### 複雑度:
- [簡単/中程度/複雑]
- 推定実装時間: [XX 分]

### 潜在的な課題:
- <any potential issues or special considerations>

### 参考実装:
- <similar existing commands in src/cli/idb/>
```

### Phase 2.5: ユーザー確認

AskUserQuestion で実装開始の確認:

**質問:** "この分析結果に基づいて実装を開始しますか?"

**選択肢:**
1. "はい、実装を開始してください" → Phase 3 に進む
2. "いいえ、分析結果のみで十分です" → 終了
3. "分析結果を見直したい（追加情報が必要）" → ユーザーの追加指示を待つ

**分岐処理:**
- 選択肢 1: Phase 3 に進み、TodoWrite で実装タスクを作成
- 選択肢 2: 以下のメッセージを表示して終了:
  ```
  分析が完了しました。実装が必要な場合は、再度このコマンドを実行してください。
  ```
- 選択肢 3: 追加の質問や確認事項をユーザーに尋ね、Phase 2 の必要なステップを再実行

### Phase 3: Rust実装生成

**注意: このフェーズは Phase 2.5 でユーザーが "はい" を選択した場合のみ実行されます。**

TodoWriteで実装タスクを作成:
```json
{
  "todos": [
    { "content": "proto/idb.protoを更新", "status": "pending", "activeForm": "proto/idb.protoを更新中" },
    { "content": "src/cli/idb/mod.rsにenumバリアント追加", "status": "pending", "activeForm": "enumバリアントを追加中" },
    { "content": "src/cli/idb/<command>.rsを作成", "status": "pending", "activeForm": "実装ファイルを作成中" },
    { "content": "src/grpc/client.rsにgRPCメソッド追加", "status": "pending", "activeForm": "gRPCメソッドを追加中" },
    { "content": "src/main.rsにルーティング追加", "status": "pending", "activeForm": "ルーティングを追加中" },
    { "content": "tests/<command>_integration.rsを作成", "status": "pending", "activeForm": "統合テストを作成中" },
    { "content": "cargo clippyでlintチェック", "status": "pending", "activeForm": "lintチェック中" },
    { "content": "cargo testでテスト実行", "status": "pending", "activeForm": "テスト実行中" }
  ]
}
```

**Step 3.1: proto/idb.proto を更新**
- 現在のprotoに存在しないメッセージがある場合のみ
- `idb/proto/idb.proto` から必要なメッセージ定義を**完全にコピー**
- フィールド番号は**絶対に変更しない**（wire互換性のため）
- service定義にRPCメソッドを追加

例:
```protobuf
service CompanionService {
  // 既存のメソッド...

  rpc screenshot(ScreenshotRequest) returns (ScreenshotResponse) {}
}

message ScreenshotRequest {
  ScreenshotFormat format = 1;
}

message ScreenshotResponse {
  bytes image_data = 1;
  ScreenshotFormat image_format = 2;
}
```

- 更新後、最初のタスクを `completed` にマーク
- `cargo build` を実行して、protoコンパイルを確認

**Step 3.2: src/cli/idb/mod.rs を更新**

ファイル先頭にモジュール宣言を追加:
```rust
pub mod <command>;
```

enum `IdbCommands` に新しいバリアントを追加:
```rust
/// <Pythonのdescription>
CommandName {
    /// <引数の説明>
    arg_name: Type,

    /// <オプションの説明>
    #[arg(long)]
    optional_arg: Option<Type>,

    /// <フラグの説明>
    #[arg(short, long)]
    flag: bool,

    // UDID指定が必要な場合
    #[arg(short, long)]
    udid: Option<String>,
},
```

- Pythonの引数名をRustのスネークケースに変換（例: `dest_path` → `dest_path`）
- フラグは `store_true` → `bool`
- 位置引数は必須、オプション引数は `Option<T>`

- タスクを `completed` にマーク
- `cargo build` で確認

**Step 3.3: src/cli/idb/<command>.rs を作成**

テンプレート:
```rust
use crate::companion::CompanionState;
use crate::grpc::IdbClient;
use crate::types::Address;

pub async fn run(
    // Pythonの引数に対応
    arg1: Type1,
    arg2: Type2,
    udid: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Companionの解決
    let state = CompanionState::default();

    let companion = if let Some(udid) = udid {
        // UDID指定がある場合
        state
            .companions()
            .find(|c| c.target_description().udid == udid)
            .ok_or(format!("No companion found for UDID: {}", udid))?
    } else {
        // ManagementCommandの場合は全companion、ClientCommandの場合は最初の1つ
        state.companions().next().ok_or("No companions available")?
    };

    // 2. gRPC接続
    let address = companion.address().ok_or("No valid address")?;
    let mut client = match &address {
        Address::DomainSocket { path } => IdbClient::connect_uds(path).await?,
        Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await?,
    };

    // 3. gRPCメソッド呼び出し
    let result = client.method_name(arg1, arg2).await?;

    // 4. 結果の出力
    // JSON出力の場合:
    println!("{}", serde_json::to_string(&result)?);

    // テキスト出力の場合:
    // println!("{}", result);

    // バイナリ出力の場合（例: screenshot）:
    // use std::io::Write;
    // std::io::stdout().write_all(&result)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // ユニットテスト（必要な場合のみ）
}
```

実装のポイント:
- **ClientCommand**: 単一companionに接続、UDIDオプションをサポート
- **ManagementCommand**: 複数companionに接続、またはmanager操作
- **エラーハンドリング**: `?` 演算子で伝播、`Box<dyn Error>` で返す
- **出力フォーマット**: Pythonと一致させる

参考実装:
- `src/cli/idb/list_targets.rs`: ManagementCommand、JSON/人間可読形式
- `src/cli/idb/launch.rs`: ClientCommand、ストリーミング、シグナルハンドリング

- タスクを `completed` にマーク
- `cargo build` で確認

**Step 3.4: src/grpc/client.rs にメソッド追加**

`impl IdbClient` ブロックに新しいメソッドを追加:

```rust
pub async fn method_name(
    &mut self,
    arg: Type,
) -> Result<ResponseType, Box<dyn std::error::Error + Send + Sync>> {
    let request = tonic::Request::new(proto::RequestType {
        field: arg,
        // ...
    });

    let response = self.client.method_name(request).await?;
    Ok(response.into_inner())
}
```

ストリーミングの場合（例: log, video）:
```rust
pub async fn method_name(
    &mut self,
    arg: Type,
) -> Result<impl Stream<Item = Result<ResponseType, tonic::Status>>, Box<dyn std::error::Error + Send + Sync>> {
    let request = tonic::Request::new(proto::RequestType {
        field: arg,
    });

    let response_stream = self.client.method_name(request).await?;
    Ok(response_stream.into_inner())
}
```

- タスクを `completed` にマーク
- `cargo build` で確認

**Step 3.5: src/main.rs にルーティング追加**

`match cli.command` の `Commands::Idb { command }` 内、`match command` に追加:

```rust
IdbCommands::CommandName { arg1, arg2, flag, udid } => {
    cli::idb::command_name::run(arg1, arg2, flag, udid).await?;
}
```

- タスクを `completed` にマーク
- `cargo build` で確認

**Step 3.6: tests/<command>_integration.rs を作成** (`--skip-tests` でない場合)

テンプレート:
```rust
use std::process::Command;

#[test]
fn test_<command>_matches_idb() {
    // Skip if idb not available
    if Command::new("idb").arg("--version").output().is_err() {
        eprintln!("Skipping test: idb not found in PATH");
        return;
    }

    // Get Python idb output
    let idb_output = Command::new("idb")
        .args(["<command>", /* test args */])
        .output()
        .expect("idb command failed");

    if !idb_output.status.success() {
        eprintln!("Skipping test: idb command failed");
        return;
    }

    // Get Rust agent-mobile output
    let rust_output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "<command>", /* test args */])
        .output()
        .expect("agent-mobile command failed");

    assert!(rust_output.status.success(), "agent-mobile command failed");

    // Compare outputs
    let idb_out = String::from_utf8_lossy(&idb_output.stdout);
    let rust_out = String::from_utf8_lossy(&rust_output.stdout);

    // Normalize and compare
    assert_eq!(normalize(&idb_out), normalize(&rust_out));
}

#[test]
fn test_<command>_error_handling() {
    // Test with invalid inputs
    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "<command>", /* invalid args */])
        .output()
        .expect("agent-mobile command failed to run");

    assert!(!output.status.success(), "Should fail with invalid args");
}

fn normalize(s: &str) -> Vec<String> {
    let mut lines: Vec<String> = s.lines().map(|l| l.trim().to_string()).collect();
    lines.sort();
    lines
}
```

バイナリ出力の場合（screenshot）:
```rust
#[test]
fn test_screenshot_creates_valid_image() {
    use std::fs;
    use std::path::Path;

    let output_path = "/tmp/test_screenshot.png";

    let output = Command::new("./target/debug/agent-mobile")
        .args(["idb", "screenshot", output_path])
        .output()
        .expect("screenshot command failed");

    assert!(output.status.success());
    assert!(Path::new(output_path).exists());

    // Verify it's a valid PNG
    let data = fs::read(output_path).unwrap();
    assert!(data.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG magic number

    // Cleanup
    fs::remove_file(output_path).ok();
}
```

- タスクを `completed` にマーク

### Phase 4: ビルドとLint

**Step 4.1: cargo clippy を実行**
```bash
cargo clippy -- -D warnings
```

警告が出た場合:
- 警告内容を分析
- 修正を適用:
  - 未使用変数: `_` プレフィックスまたは削除
  - 不要な借用: `&` を削除
  - 不要なクローン: `.clone()` を削除
  - など
- 再度 `cargo clippy` を実行
- 最大3回まで自動修正を試行
- それでも失敗する場合はユーザーに相談

- タスクを `completed` にマーク

### Phase 5: テスト実行 (`--skip-tests` でない場合)

**Step 5.1: 統合テストを実行**
```bash
cargo test --test <command>_integration
```

テスト失敗時:
1. 出力差異を表示
2. 原因を分析:
   - 出力フォーマットの違い
   - ソート順の違い
   - JSONフィールドの欠落/追加
   - バイナリデータの違い
3. 実装を修正
4. 再度テスト実行
5. 最大3回まで修正を試行
6. それでも失敗する場合:
   ```
   テストが失敗しました。

   Python idb出力:
   <idb output>

   Rust agent-mobile出力:
   <rust output>

   差異:
   <diff>

   この差異は許容可能ですか？または修正が必要ですか？
   ```

- タスクを `completed` にマーク

**Step 5.2: フルテストスイートを実行**
```bash
cargo test
```

全テストが通ることを確認。

### Phase 6: サマリー生成

完了時に以下のサマリーを生成:

```markdown
## Rust実装完了: <command-name>

### 作成/修正ファイル:
- ✓ proto/idb.proto (XXメッセージ追加)
- ✓ src/cli/idb/mod.rs (enumバリアント追加)
- ✓ src/cli/idb/<command>.rs (新規作成、XX行)
- ✓ src/grpc/client.rs (XXメソッド追加)
- ✓ src/main.rs (ルーティング追加)
- ✓ tests/<command>_integration.rs (新規作成、XXテスト)

### 実装詳細:
- コマンドタイプ: [ClientCommand/ManagementCommand]
- 引数: <list>
- gRPCメソッド: <list>
- 特殊処理: <any special cases>

### テスト結果:
- ビルド: ✓ 成功
- Clippy: ✓ 警告なし
- 統合テスト: ✓ XXテストパス

### 使用例:
```bash
agent-mobile idb <command> [arguments]
```

### 次のステップ:
- 実機/シミュレータで手動テスト
- エッジケースのテスト追加を検討
```

## 重要な注意事項

1. **Proto互換性**: フィールド番号は絶対に変更しない
2. **段階的ビルド**: 各ファイル修正後に `cargo build` を実行
3. **Python互換性**: 統合テストで完全互換性を保証
4. **TodoWrite**: 各タスク完了時に即座にステータス更新
5. **エラーハンドリング**: 3回まで自動修正、それ以降はユーザー相談

## コマンドタイプ別の実装パターン

### ClientCommand (例: screenshot, launch)
- 単一デバイスへの操作
- UDIDオプションをサポート
- companion解決が必要

### ManagementCommand (例: list-targets, kill)
- 複数デバイス管理またはdaemon操作
- 全companionへの接続
- またはcompanion管理操作

### 複雑なケース
- **サブコマンド**: 別のenumを作成（例: AppCommands）
- **ストリーミング**: `Stream` を使用、`tokio_stream` を活用
- **バイナリデータ**: `Vec<u8>` で扱い、`std::io::Write` で出力
- **シグナルハンドリング**: `tokio::signal` を使用

## トラブルシューティング

### ビルドエラー: "message not found"
- `proto/idb.proto` にメッセージ定義が不足
- `idb/proto/idb.proto` から完全にコピー

### テストエラー: "No such file or directory"
- テストバイナリがビルドされていない: `cargo build` を実行
- Python idbがインストールされていない: `pip install fb-idb` を実行

### テストエラー: "No companions available"
- シミュレータが起動していない: `xcrun simctl boot <UDID>` を実行
- companionが実行されていない: `idb_companion --udid <UDID>` を実行

このスキルを使用して、残り32個のidbコマンドを効率的にRustに移植してください！
