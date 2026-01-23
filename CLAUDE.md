 ## 技術スタック

 - **Rust 2021**: メインアプリケーション
 - **tokio 1.49**: 非同期ランタイム
 - **tonic 0.12 + prost 0.13**: gRPC クライアント/Protocol Buffers
 - **clap 4.5**: CLI フレームワーク (derive)
 - **idb_companion**: Swift/ObjC ダエモン（外部プロセス）
 - **Python idb**: 参照実装（サブモジュール）

 ## 開発ガイド

### 開発フロー

`agent-mobile <feature>` の変更時は、以下のサイクルで開発を進めます：

```
実装 → ビルド → ユニットテスト → 統合テスト → 実機動作確認 → コミット
```

**重要**: 実機動作確認は必須ステップです。コードが正しくビルドできても、実際のシミュレータ/デバイスで期待通りに動作することを確認する必要があります。

#### 基本的な開発サイクル

```bash
# 1. 機能実装
# src/ 配下のファイルを編集

# 2. ビルド検証
cargo build --verbose

# 3. ユニットテスト
cargo test --verbose --bins

# 4. 実機動作確認 (必須！)
# → 次のセクション「実機動作確認（必須）」を参照

# 5. 統合テスト (test-setup実行済みの場合)
cargo test --test cli -- --test-threads=1

# 6. コミット
git add .
git commit -m "feat: 変更内容の説明"
```

**ポイント**:
- **実機確認なしでのコミットは禁止**: ビルドが通っても、実際の動作を確認するまでコミットしないでください

### 実機動作確認（必須）

#### iOS の動作確認手順

```bash
/mobile-e2e ios
```

#### Android の動作確認手順

```bash
# 1. テスト環境起動
/mobile-e2e android
```

#### 動作確認チェックリスト

各機能変更後、以下を必ず確認してください：

- [ ] コマンドが期待通りの動作をする
- [ ] エラーメッセージが適切に表示される
- [ ] UI操作の結果が視覚的に確認できる
- [ ] スクリーンショットで証跡を保存した
- [ ] Python idbとの動作差異がない (該当する場合)

### ベストプラクティス

#### テスト駆動開発 (TDD)

新機能開発時は、以下の順序を推奨します：

```bash
# 1. テストケースを先に書く
# tests/cli/your_feature_test.rs を作成

# 2. 実装する
# src/platform/ios/grpc/your_feature.rs を作成

# 3. テストを実行して確認
cargo test --test cli your_feature -- --test-threads=1

# 4. 実機で動作確認
agent-mobile <your-command>
```

#### ヘルパー関数の活用

`tests/idb/common/mod.rs` には、テストで使える便利な関数があります：

```rust
// CompanionClient の取得
let client = common::get_client().await?;

// テストアプリの起動
common::launch_test_app(&client).await?;

// スクリーンショットの取得
let screenshot = common::take_screenshot(&client).await?;

// ファイル操作
common::push_file(&client, "/path/to/source", "/path/to/dest").await?;
```

#### ドキュメント更新

新機能追加時は、以下のドキュメントも更新してください：

- **CLAUDE.md**: AIエージェント向けの使用例
- **README.md**: ユーザー向けクイックスタート
- **docs/ARCHITECTURE.md**: アーキテクチャへの影響
- **proto/idb.proto**: gRPC API の変更 (該当する場合)

#### AIエージェント対応

Claude Code などの AIエージェントが効率的に使えるように、以下を心がけてください：

- **JSON出力対応**: `--json` フラグで JSON 形式の出力をサポート
- **明確なエラーメッセージ**: エラー時に何が問題か、どう解決するかを明示
- **ヘルプの充実**: `--help` で十分な情報を提供
- **冪等性**: 同じコマンドを複数回実行しても安全

#### コード検索

**重要**: コードベース内の検索には必ず `ck` CLI を使用してください。`grep`、`pgrep`、`rg` などの従来の検索ツールは使用しないでください。

##### ck CLI を使う理由

- **セマンティック検索**: 意味的に関連するコードを発見
- **コンテキスト理解**: 関数名や変数名だけでなく、実装の意図を理解
- **効率性**: 大規模コードベースでも高速
- **AI最適化**: Claude Code などの AIエージェント向けに設計

##### 使用方法

```bash
# セマンティック検索（推奨）
/ck "error handling"
/ck "grpc client implementation"

# 具体的な関数や型を探す場合
/ck "pub struct IdbClient"
/ck "async fn launch_app"

# ファイル名で検索
/ck "mod.rs"
```

##### grep/pgrep を使わない理由

- **文字列マッチのみ**: 意味的な関連性を理解できない
- **ノイズが多い**: 無関係な結果が大量に表示される
- **コンテキスト不足**: コードの意図や目的を把握できない