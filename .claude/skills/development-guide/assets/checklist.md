# agent-mobile 機能実装チェックリスト

新しいCLIコマンドを実装する際の完全チェックリスト。

## フェーズ0: 環境セットアップ（初回のみ）

### 開発環境確認

**環境一括チェック（推奨）:**
- [ ] `cargo run -- doctor` で環境診断実行
- [ ] エラー/警告がないことを確認

**iOS開発の場合:**
- [ ] `./.claude/skills/development-guide/scripts/setup-ios.sh` 実行成功
- [ ] Xcode Command Line Tools インストール確認
- [ ] XCUITest Runner インストール確認
- [ ] シミュレータ起動確認
- [ ] `agent-mobile device list` でデバイス検出確認

**Android開発の場合:**
- [ ] `./.claude/skills/development-guide/scripts/setup-android.sh` 実行成功
- [ ] Android SDK (adb, emulator) インストール確認
- [ ] adb server 起動確認 (TCP :5037)
- [ ] エミュレータ起動確認
- [ ] `agent-mobile device list` でデバイス検出確認

**トラブルシューティング:**
- [ ] エラーが出た場合 → `references/environment-setup.md` 参照

---

## フェーズ1: 設計

### プラットフォーム判断

**iOS実装判断:**
- [ ] `./scripts/platform-check.sh <feature>` で実装方法確認
- [ ] XCUITest Runner (HTTP)で実装可能 → with_xcuitest() パターン
- [ ] simctl操作（デバイスライフサイクル） → xcrun simctl実装
- [ ] 両方で可能 → ハイブリッド実装（XCUITest Runner優先、fallback）

**Android実装判断:**
- [ ] ADB native protocol で実現可能か確認
- [ ] 複雑な操作の場合はGateway層の AndroidDevice 拡張

### 引数設計

- [ ] コマンド固有の引数を定義
- [ ] `DeviceArgs` flatten を使用（--udid、platformは自動検出）
- [ ] 必要に応じて `DeviceFormatArgs` を使用（--udid + --format）
- [ ] 引数の検証ロジックを考慮

### アーキテクチャ配置

- [ ] コア操作 → `src/core/<name>.rs`
- [ ] 管理操作 → `src/<name>.rs`
- [ ] Platform層の拡張が必要か確認

## フェーズ2: 実装

### ファイル作成

- [ ] テンプレート生成: `./scripts/new-command.sh <command>`
- [ ] または手動作成:
  - [ ] `src/core/<name>.rs` 作成
  - [ ] `tests/cli/<name>_integration.rs` 作成
  - [ ] `src/command.rs` に追加:
    - [ ] `Commands::<Name>(crate::core::<name>::<Name>Args)`
  - [ ] `src/main.rs` の match 分岐に追加:
    - [ ] `Commands::<Name>(args) => crate::core::<name>::run(args).await`

### iOS実装（XCUITest Runner HTTP）

- [ ] `with_xcuitest()` パターン使用（UDIDパラメータなし）
- [ ] プラットフォーム検出を先に実行:
  ```rust
  let platform = match args.device.udid.as_deref() {
      Some(udid) => crate::device::detect_platform_from_udid(udid).await?,
      None => DeviceResolver::detect_platform().await?,
  };
  ```
- [ ] エラーハンドリング（`?` 演算子）
- [ ] 適切なXCUITestClient HTTPメソッド呼び出し

### iOS実装（xcrun simctl）

- [ ] `crates/platform-ios/src/simctl/management.rs` に関数追加
- [ ] `Command::new("xcrun")` でsimctl呼び出し
- [ ] 出力パース、エラーハンドリング

### Android実装

- [ ] `crates/platform-android/src/adb/` に実装
- [ ] ADB native protocol (adb_client crate) 使用
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
- [ ] `OutputFormat::Text` で人間可読出力

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
  - [ ] `ensure_device_ready()`
  - [ ] `assert_success()`

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
- [ ] 複数デバイスで動作確認（該当する場合）

**実機確認なしでのコミットは禁止!**

## フェーズ5: ドキュメント

### コード内ドキュメント

- [ ] モジュールレベルdocコメント (`//!`)
- [ ] 公開関数にdocコメント
- [ ] 使用例を含む
- [ ] TODOコメント削除

### プロジェクトドキュメント更新

- [ ] **CLAUDE.md**: AI開発者向け使用例追加
- [ ] **README.md**: ユーザー向け使用例追加（該当する場合）
- [ ] XCUITest Runner API変更の確認（該当する場合）

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
