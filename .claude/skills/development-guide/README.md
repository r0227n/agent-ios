# agent-mobile CLI開発スキル

agent-mobile CLI（Rust製モバイルE2Eテストツール）の新機能開発、既存機能リファクタリングのための包括的な開発ガイドスキル。

## 概要

このスキルは、agent-mobile CLIの開発プロセスを体系化し、AI開発者が効率的に新機能を追加できるようにします。

**提供内容:**
- 4層アーキテクチャ（CLI/Gateway/Platform/Core）の理解促進
- プラットフォーム実装判断（iOS: XCUITest Runner HTTP vs xcrun simctl、Android: ADB native protocol）の支援
- 必須開発フロー（実装→テスト→実機確認→コミット）の遵守
- 頻出パターン（with_xcuitest、DeviceArgs、セッション管理）のテンプレート化
- UI スナップショット（@e1, @e2 ref）のアーキテクチャ

## ディレクトリ構造

```
.claude/skills/development-guide/
├── SKILL.md                          # メインスキル定義
├── scripts/                          # 開発支援スクリプト
│   ├── setup-ios.sh                  # iOS環境自動セットアップ
│   ├── setup-android.sh              # Android環境自動セットアップ
│   ├── new-command.sh                # CLIコマンドテンプレート生成
│   ├── platform-check.sh             # iOS実装判断支援
│   └── run-tests.sh                  # 3層テスト実行ガイド
├── assets/                           # テンプレートファイル
│   ├── command-template.rs           # CLIコマンド基本構造
│   ├── test-template.rs              # 統合テスト構造
│   └── checklist.md                  # 対話的実装チェックリスト
├── references/                       # 詳細リファレンス
│   ├── environment-setup.md          # 環境セットアップ詳細ガイド
│   ├── implementation-patterns.md    # コーディングパターン集
│   ├── testing-guide.md              # テスト戦略詳細
│   └── architecture.md               # アーキテクチャ詳細
└── README.md                         # このファイル
```

## 使用方法

### 1. メインスキル参照

新しいCLIコマンドを追加する際、まず `SKILL.md` を参照してください。

**SKILL.md の構成:**
- Overview: agent-mobile概要、CLIコマンド一覧、4層アーキテクチャ図
- Quick Start Decision Tree: 新機能追加時の判断フロー
- Development Workflow: 必須6ステップ（設計→実装→テスト→実機確認→コミット）
- Common Patterns: 頻出パターンのクイックリファレンス（with_xcuitest、DeviceArgs等）
- Platform Decision (iOS): XCUITest Runner (HTTP) vs xcrun simctl の判断基準
- Platform Decision (Android): ADB native protocol 構成
- Scripts & Templates: スクリプト使用方法
- References: 詳細リファレンスへのリンク

### 2. 環境セットアップ（初回）

機能開発を開始する前に、開発環境を準備してください。

#### 環境一括チェック（推奨）

```bash
cargo run -- doctor
```

#### iOS環境セットアップ

```bash
./scripts/setup-ios.sh
```

#### Android環境セットアップ

```bash
./scripts/setup-android.sh
```

**詳細**: `references/environment-setup.md` 参照

### 3. スクリプト使用

#### 3.1. platform-check.sh - iOS実装判断支援

新機能のiOS実装方法を判断します（XCUITest Runner or simctl）。

```bash
./scripts/platform-check.sh <feature-name>
```

**例:**
```bash
./scripts/platform-check.sh accessibility
# → Recommendation: Use XCUITest Runner (HTTP)

./scripts/platform-check.sh boot
# → Recommendation: Use xcrun simctl
```

#### 3.2. new-command.sh - CLIコマンドテンプレート生成

新しいCLIコマンドの骨格を自動生成します。

```bash
./scripts/new-command.sh <command-name> [description]
```

**生成内容:**
- `src/core/<command-name>.rs`: コマンド実装骨格（`assets/command-template.rs` ベース）
- `tests/cli/<command-name>_integration.rs`: 統合テスト骨格（`assets/test-template.rs` ベース）
- `src/core/mod.rs`: 自動的にmod宣言を追加

**手動追加が必要:**
- `src/command.rs` の Commands enum にバリアント追加
- `src/main.rs` の match 分岐に追加

#### 3.3. run-tests.sh - 3層テスト実行ガイド

テスト実行を順次ガイドします。

```bash
./scripts/run-tests.sh [test-name]
```

### 4. リファレンス参照

詳細な情報が必要な場合、`references/` ディレクトリ内のファイルを参照してください。

#### 4.1. environment-setup.md

環境セットアップの詳細ガイド（Phase 0）。

**内容:**
- iOS/Android環境要件
- ツールインストール手順（Xcode、XCUITest Runner、Android SDK）
- シミュレータ/エミュレータ管理
- トラブルシューティング

#### 4.2. implementation-patterns.md

コーディングパターン集。

**内容:**
- `with_xcuitest()` パターン（UDIDパラメータなし）
- 引数パターン（DeviceArgs、DeviceFormatArgs）
- エラーハンドリングパターン
- JSON出力パターン
- 非同期処理パターン

#### 4.3. testing-guide.md

テスト戦略詳細。

**内容:**
- 3層テスト詳細（ユニット、統合、実機確認）
- `tests/cli/common/mod.rs` ヘルパー関数リスト
- 実機確認詳細手順（/mobile-e2eスキル使用）
- TDDサイクル実践例

#### 4.4. architecture.md

アーキテクチャ詳細。

**内容:**
- 4層アーキテクチャ詳細（各層の責務、実装例）
- モジュール配置規則
- データフローパターン
- Cargoワークスペース依存関係グラフ

### 5. チェックリスト活用

実装時に `assets/checklist.md` を参照し、必要なステップを確認してください。

**チェックリストの構成:**
- フェーズ0: 環境セットアップ（doctor コマンド、iOS/Android環境確認）
- フェーズ1: 設計（プラットフォーム判断、引数設計、アーキテクチャ配置）
- フェーズ2: 実装（ファイル作成、iOS/Android実装、エラーハンドリング）
- フェーズ3: テスト（ユニット、統合テスト作成・実行）
- フェーズ4: 実機動作確認（iOS/Android確認、チェックリスト）
- フェーズ5: ドキュメント（コード内、プロジェクト、変更履歴）
- フェーズ6: コミット（最終チェック、コミット）

## クイックスタート

新機能追加の最速フロー:

```bash
# 0. 環境確認（初回のみ）
cargo run -- doctor
# または
./scripts/setup-ios.sh  # ./scripts/setup-android.sh

# 1. 実装判断
./scripts/platform-check.sh <feature>

# 2. テンプレート生成
./scripts/new-command.sh <command> "<description>"

# 3. 実装
# src/core/<command>.rs の TODOコメントを埋める
# src/command.rs に Commands enum バリアント追加
# src/main.rs に match 分岐追加

# 4. ビルド & ユニットテスト
cargo build
cargo test --verbose --bins

# 5. 統合テスト
cargo test --test cli <command> -- --test-threads=1

# 6. 実機確認（必須!）
/mobile-e2e ios
agent-mobile <command> [args]
agent-mobile screenshot /tmp/<command>_evidence.png

# 7. コミット
git add .
git commit -m "feat: add <command> command"
```

## トラブルシューティング

### スクリプトが動作しない

**問題**: `./scripts/new-command.sh: Permission denied`

**解決**:
```bash
chmod +x .claude/skills/development-guide/scripts/*.sh
```

### 実機確認でコマンドが見つからない

**問題**: `agent-mobile: command not found`

**解決**:
```bash
cargo build
./target/debug/agent-mobile <command> [args]
```

### 環境診断で問題が見つかった

**解決**:
```bash
cargo run -- doctor --format json
# → JSON出力で詳細なエラー情報を確認
```

## 開発フローの原則

### 必須ステップ: 実機動作確認

**最重要**: ビルド成功 ≠ 正しい動作

実機確認は**必須ステップ**です。ビルドが通っても、実際の動作を確認するまでコミットしないでください。

### Progressive Disclosure

- **SKILL.mdは簡潔に**: コア開発フローと判断ツリーのみ
- **詳細はreferences/へ**: パターンの複数例、詳細説明、網羅的リスト
- **文脈連動リンク**: 各判断ポイントから適切なreferenceへ誘導

## 関連ドキュメント

**プロジェクト内:**
- `CLAUDE.md`: AI開発者向けクイックスタート
- `README.md`: ユーザー向け使用方法
- `crates/xcuitest-runner/ARCHITECTURE.md`: XCUITest Runner アーキテクチャ

**このスキル内:**
- `SKILL.md`: メインスキル定義
- `references/environment-setup.md`: 環境セットアップ詳細
- `references/implementation-patterns.md`: コーディングパターン集
- `references/testing-guide.md`: テスト戦略詳細
- `references/architecture.md`: アーキテクチャ詳細

---

**まとめ**: 新機能追加時は 環境確認（Phase 0） → `SKILL.md` → スクリプト → 実機確認（必須!） → コミット
