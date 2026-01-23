# agent-mobile 機能実装チェックリスト

新しいCLIコマンドを実装する際の完全チェックリスト。

## フェーズ1: 設計

### プラットフォーム判断

**iOS実装判断:**
- [ ] `proto/idb.proto` を確認（`./scripts/platform-check.sh <feature>`）
- [ ] RPC定義がある → idb gRPC実装
- [ ] RPC定義がない → xcrun simctl実装
- [ ] 両方で可能 → ハイブリッド実装（gRPC優先、fallback）

**Android実装判断:**
- [ ] adbコマンドで実現可能か確認
- [ ] 複雑な操作の場合はGateway層でラッパー検討

### 引数設計

- [ ] コマンド固有の引数を定義
- [ ] `DeviceArgs` flatten を使用（--udid, --platform）
- [ ] 必要に応じて `DeviceFormatArgs` を使用（--format）
- [ ] 引数の検証ロジックを考慮

### アーキテクチャ配置

- [ ] トップレベルコマンド → `src/core/<name>.rs`
- [ ] IDB互換コマンド → `src/idb/<name>.rs`
- [ ] Platform層の拡張が必要か確認

## フェーズ2: 実装

### ファイル作成

- [ ] テンプレート生成: `./scripts/new-command.sh <command>`
- [ ] または手動作成:
  - [ ] `src/core/<name>.rs` 作成
  - [ ] `tests/cli/<name>_integration.rs` 作成
  - [ ] `src/mod.rs` に追加:
    - [ ] `pub mod <name>;`
    - [ ] `Commands::<Name>(<name>::<Name>Args)`
    - [ ] `Commands::<Name>(args) => <name>::run(args).await`

### iOS実装（idb gRPC）

- [ ] `with_client()` パターン使用
- [ ] ストリーミングの場合は `with_client_streaming()` 使用
- [ ] エラーハンドリング（`?` 演算子）
- [ ] 適切なRPCメソッド呼び出し

### iOS実装（xcrun simctl）

- [ ] `crates/platform-ios/src/simctl/management.rs` に関数追加
- [ ] `Command::new("xcrun")` でsimctl呼び出し
- [ ] 出力パース、エラーハンドリング

### Android実装

- [ ] `crates/platform-android/src/adb/` に実装
- [ ] adbコマンドラッパー作成
- [ ] 出力パース、エラーハンドリング

### エラーハンドリング

- [ ] `CommandResult` 型を使用
- [ ] アクション可能なエラーメッセージ
  - 何が問題か明示
  - どう解決するか提示
- [ ] エラー連鎖（`?` 演算子）を適切に使用

### JSON出力対応（該当する場合）

- [ ] `DeviceFormatArgs` 使用
- [ ] `OutputFormat::Json` で構造化出力
- [ ] `OutputFormat::Human` で人間可読出力

## フェーズ3: テスト

### ユニットテスト

- [ ] `cargo build --verbose` でビルド確認
- [ ] `#[cfg(test)]` mod tests にテスト追加
- [ ] 引数パース、基本ロジックをテスト
- [ ] `cargo test --verbose --bins` で実行

### 統合テスト作成

- [ ] `tests/cli/<name>_integration.rs` 実装
- [ ] 正常系テスト: `test_<name>_success()`
- [ ] 異常系テスト: `test_<name>_invalid_device()`
- [ ] エッジケースのテスト追加
- [ ] `common` モジュールのヘルパー使用:
  - [ ] `get_available_udid()`
  - [ ] `ensure_companion_running()`
  - [ ] `assert_success()`, `assert_failure()`

### 統合テスト実行

- [ ] iOS: シミュレータ起動確認
- [ ] Android: エミュレータ起動確認
- [ ] `cargo test --test cli <name> -- --test-threads=1` 実行
- [ ] 全テストがパス

## フェーズ4: 実機動作確認（必須!）

### iOS確認

- [ ] `/mobile-e2e ios` でテスト環境起動
- [ ] コマンド実行: `agent-mobile <command> [args]`
- [ ] シミュレータで結果を視覚確認:
  - [ ] UI操作が期待通りに動作
  - [ ] 画面遷移が正しい
  - [ ] アニメーションが適切
- [ ] 証跡保存: `agent-mobile screenshot /tmp/<command>_evidence.png`
- [ ] 異常系確認:
  - [ ] `agent-mobile <command> --udid invalid-udid`
  - [ ] エラーメッセージが明確で実行可能
- [ ] 複数シナリオで確認（該当する場合）
- [ ] Python idbとの動作差異確認（該当する場合）

### Android確認

- [ ] `/mobile-e2e android` でテスト環境起動
- [ ] コマンド実行: `agent-mobile <command> [args]`
- [ ] エミュレータで結果を視覚確認
- [ ] 証跡保存: `agent-mobile screenshot /tmp/<command>_evidence.png`
- [ ] 異常系確認
- [ ] 複数シナリオで確認

### 確認チェックリスト

- [ ] コマンドが期待通り動作（正常系）
- [ ] エラーメッセージが適切（異常系）
- [ ] UI操作結果が視覚的に確認可能
- [ ] スクリーンショットで証跡保存済み
- [ ] Python idbとの動作差異なし（該当する場合）
- [ ] 複数デバイスで動作確認（該当する場合）

**⚠️ 実機確認なしでのコミットは禁止!**

## フェーズ5: ドキュメント

### コード内ドキュメント

- [ ] モジュールレベルdocコメント (`//!`)
- [ ] 公開関数にdocコメント
- [ ] 使用例を含む
- [ ] TODOコメント削除

### プロジェクトドキュメント更新

- [ ] **CLAUDE.md**: AI開発者向け使用例追加
- [ ] **README.md**: ユーザー向け使用例追加（該当する場合）
- [ ] **docs/ARCHITECTURE.md**: アーキテクチャへの影響記載（該当する場合）
- [ ] **proto/idb.proto**: gRPC API変更のコメント（該当する場合）

### 変更履歴

- [ ] コミットメッセージ準備:
  - `feat: add <command> command`
  - `fix: <issue description>`
  - `refactor: <refactor description>`

## フェーズ6: コミット

### 最終チェック

- [ ] 全ユニットテストがパス
- [ ] 全統合テストがパス
- [ ] 実機確認完了（iOS/Android両方、該当する場合）
- [ ] ドキュメント更新完了
- [ ] デッドコード削除
- [ ] フォーマット確認: `cargo fmt`
- [ ] リント確認: `cargo clippy` (警告なし)

### コミット

- [ ] `git add .`
- [ ] `git commit -m "feat: add <command> command"`
- [ ] プルリクエスト作成（該当する場合）

## 追加考慮事項

### パフォーマンス

- [ ] 不要なコピーを避ける
- [ ] 非同期処理を適切に使用
- [ ] ストリーミングで大量データ処理

### セキュリティ

- [ ] ユーザー入力の検証
- [ ] コマンドインジェクション対策
- [ ] パスインジェクション対策

### AI対応

- [ ] JSON出力対応（`--format json`）
- [ ] 明確なエラーメッセージ
- [ ] ヘルプテキストの充実
- [ ] 冪等性（複数回実行しても安全）

## チェックリスト完了

全てのチェックがついたら、実装完了です！

```bash
git commit -m "feat: add <command> command"
git push
```
