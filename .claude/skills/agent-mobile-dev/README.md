# agent-mobile CLI開発スキル

agent-mobile CLI（Rust製モバイルE2Eテストツール）の新機能開発、既存機能リファクタリングのための包括的な開発ガイドスキル。

## 概要

このスキルは、agent-mobile CLIの開発プロセスを体系化し、AI開発者が効率的に新機能を追加できるようにします。

**提供内容:**
- 4層アーキテクチャ（CLI/Gateway/Platform/Core）の理解促進
- プラットフォーム実装判断（iOS: idb gRPC vs xcrun simctl）の自動化
- 必須開発フロー（実装→テスト→実機確認→コミット）の遵守
- 頻出パターン（with_client、DeviceArgs）のテンプレート化

## ディレクトリ構造

```
.claude/skills/agent-mobile-dev/
├── SKILL.md                          # メインスキル定義（630行）
├── scripts/                          # 開発支援スクリプト
│   ├── setup-ios.sh                  # iOS環境自動セットアップ
│   ├── setup-android.sh              # Android環境自動セットアップ
│   ├── new-command.sh                # CLIコマンドテンプレート生成
│   ├── platform-check.sh             # iOS実装判断支援（proto確認）
│   └── run-tests.sh                  # 3層テスト実行ガイド
├── assets/                           # テンプレートファイル
│   ├── command-template.rs           # CLIコマンド基本構造
│   ├── test-template.rs              # 統合テスト構造
│   └── checklist.md                  # 対話的実装チェックリスト
├── references/                       # 詳細リファレンス
│   ├── environment-setup.md          # 環境セットアップ詳細ガイド
│   ├── platform-decisions.md         # iOS実装判断基準詳細（572行）
│   ├── implementation-patterns.md    # コーディングパターン集（685行）
│   ├── testing-guide.md              # テスト戦略詳細（784行）
│   └── architecture.md               # アーキテクチャ詳細（712行）
└── README.md                         # このファイル
```

## 使用方法

### 1. メインスキル参照

新しいCLIコマンドを追加する際、まず `SKILL.md` を参照してください。

```bash
# VS Codeで開く
code .claude/skills/agent-mobile-dev/SKILL.md

# less で参照
less .claude/skills/agent-mobile-dev/SKILL.md
```

**SKILL.md の構成:**
- Overview: agent-mobile概要、4層アーキテクチャ図
- Quick Start Decision Tree: 新機能追加時の判断フロー
- Development Workflow: 必須5ステップ（設計→実装→テスト→実機確認→コミット）
- Common Patterns: 頻出パターンのクイックリファレンス
- Platform Decision (iOS): idb gRPC vs xcrun simctl の判断基準
- Scripts & Templates: スクリプト使用方法
- References: 詳細リファレンスへのリンク

### 2. 環境セットアップ（初回）

機能開発を開始する前に、開発環境を準備してください。

#### 2.1. iOS環境セットアップ

```bash
./scripts/setup-ios.sh
```

**実行内容:**
1. Xcode & xcrun simctl確認
2. idb_companionインストール確認
3. シミュレータ起動（未起動の場合）
4. agent-mobile接続確認

#### 2.2. Android環境セットアップ

```bash
./scripts/setup-android.sh
```

**実行内容:**
1. Android SDK (adb, emulator)確認
2. adb server起動
3. エミュレータ起動（未起動の場合）
4. agent-mobile接続確認

**詳細**: `references/environment-setup.md` 参照

### 3. スクリプト使用

#### 3.1. platform-check.sh - iOS実装判断支援

新機能のiOS実装方法を判断します（proto/idb.proto を検索）。

```bash
./scripts/platform-check.sh <feature-name>
```

**例:**
```bash
# アクセシビリティ機能の実装判断
./scripts/platform-check.sh accessibility

# 出力:
# ✓ Found RPC definition(s):
#     rpc accessibility_info(AccessibilityInfoRequest) returns (AccessibilityInfoResponse) {}
#
# Recommendation: Use idb gRPC
# Implementation:
#   - Use with_client() pattern
#   - Location: src/core/<feature>.rs or src/idb/<feature>.rs
# ...
```

```bash
# クリップボード機能の実装判断
./scripts/platform-check.sh clipboard

# 出力:
# ✗ No RPC found for: clipboard
#
# Recommendation: Use xcrun simctl
# Example (simctl command):
#   xcrun simctl pbcopy <udid> "text"
# ...
```

#### 3.2. new-command.sh - CLIコマンドテンプレート生成

新しいCLIコマンドの骨格を自動生成します。

```bash
./scripts/new-command.sh <command-name> [description]
```

**例:**
```bash
./scripts/new-command.sh vibrate "Vibrate device"

# 生成:
# ✓ Created src/core/vibrate.rs
# ✓ Created tests/cli/vibrate_integration.rs
# ✓ Added 'pub mod vibrate;' to src/mod.rs
# ⚠ Please manually add 'Vibrate(vibrate::VibrateArgs),' to Commands enum in src/mod.rs
# ⚠ Please manually add 'Commands::Vibrate(args) => vibrate::run(args).await,' to match in src/mod.rs
#
# Next steps:
#   1. cargo build
#   2. Customize vibrate.rs implementation
#   3. cargo test --test cli vibrate
#   4. /mobile-e2e ios  # Real device verification (REQUIRED!)
```

**生成内容:**
- `src/core/<command-name>.rs`: コマンド実装骨格（`assets/command-template.rs` ベース）
- `tests/cli/<command-name>_integration.rs`: 統合テスト骨格（`assets/test-template.rs` ベース）
- `src/mod.rs`: 自動的にmod宣言を追加（Commands enum、match分岐は手動）

#### 3.3. run-tests.sh - 3層テスト実行ガイド

テスト実行を順次ガイドします。

```bash
./scripts/run-tests.sh [test-name]
```

**出力例:**
```
==========================================
 agent-mobile Test Execution Guide
==========================================

Step 1: Unit Tests
-------------------
Description: Tests internal logic without external dependencies
Location: #[cfg(test)] mod tests within src/ files

Command:
  cargo test --verbose --bins

Step 2: Integration Tests
-------------------------
Description: Tests CLI commands with real devices/simulators
Location: tests/cli/<name>_integration.rs

Prerequisites:
  - iOS: Simulator running with idb_companion
  - Android: Emulator running with adb

Command:
  cargo test --test cli -- --test-threads=1

Step 3: Real Device Verification (REQUIRED!)
---------------------------------------------
Description: Manual verification on actual device/simulator
Purpose: Ensure visual behavior matches expectations

⚠️  CRITICAL: Do NOT commit without completing this step!
    Build success ≠ Correct behavior

iOS Verification:
  1. Start test environment:
     $ /mobile-e2e ios
  2. Run command:
     $ agent-mobile <command> [args]
  ...
```

### 4. リファレンス参照

詳細な情報が必要な場合、`references/` ディレクトリ内のファイルを参照してください。

#### 4.1. environment-setup.md

環境セットアップの詳細ガイド（Phase 0）。

**内容:**
- iOS/Android環境要件
- ツールインストール手順（Xcode、idb_companion、Android SDK）
- シミュレータ/エミュレータ管理
- トラブルシューティング（デバイス検出、起動エラーなど）
- セットアップスクリプトの詳細

#### 4.2. platform-decisions.md

iOS実装判断基準の詳細版（`.claude/rules/cli-feature.md` を統合・拡充）。

**内容:**
- 判断基準、判断フローチャート詳細版
- idb.proto RPC一覧（カテゴリ別: App, HID, File, Media, Debug...）
- xcrun simctl コマンド一覧
- ハイブリッド実装パターン（複数例）
- 機能別実装状況表
- 判断履歴（既存機能がなぜgRPC/simctlを選んだか）

#### 4.3. implementation-patterns.md

コーディングパターン集。

**内容:**
- `with_client()` の複数バリエーション（基本形、ストリーミング用、複数操作）
- 引数パターン（DeviceArgs、DeviceFormatArgs、カスタム検証）
- エラーハンドリングパターン（アクション可能なエラーメッセージ）
- JSON出力パターン（構造化出力、エラー時の処理）
- 非同期処理パターン（tokio::select!、ストリーミング応答）

#### 4.4. testing-guide.md

テスト戦略詳細。

**内容:**
- 3層テスト詳細（ユニット、統合、実機確認）
- `tests/cli/common/mod.rs` ヘルパー関数リスト
- `tests/idb/common/mod.rs` ヘルパー関数リスト
- **実機確認詳細手順（/mobile-e2eスキル使用）**
  - iOS/Android別の確認手順
  - スクリーンショット保存方法
  - 確認チェックリスト
- TDDサイクル実践例

#### 4.5. architecture.md

アーキテクチャ詳細（`docs/ARCHITECTURE.md` の補足）。

**内容:**
- 4層アーキテクチャ詳細（各層の責務、実装例）
- モジュール配置規則（コマンド、Platform実装、ヘルパー）
- データフローパターン（単純なコマンド、ストリーミング、JSON出力）
- Cargoワークスペース依存関係グラフ
- 設計原則（関心の分離、依存性逆転、単一責任など）

### 5. チェックリスト活用

実装時に `assets/checklist.md` を参照し、必要なステップを確認してください。

```bash
# チェックリストを開く
code .claude/skills/agent-mobile-dev/assets/checklist.md

# または印刷してチェック
cat .claude/skills/agent-mobile-dev/assets/checklist.md
```

**チェックリストの構成:**
- フェーズ0: 環境セットアップ（iOS/Android環境確認、デバイス検出）
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
./scripts/setup-ios.sh  # または ./scripts/setup-android.sh

# 1. 実装判断
./scripts/platform-check.sh <feature>

# 2. テンプレート生成
./scripts/new-command.sh <command> "<description>"

# 3. 実装
# src/core/<command>.rs の TODOコメントを埋める

# 4. ビルド & ユニットテスト
cargo build
cargo test --verbose --bins

# 5. 統合テスト
cargo test --test cli <command> -- --test-threads=1

# 6. 実機確認（必須!）
/mobile-e2e ios
agent-mobile <command> [args]
# → シミュレータで視覚確認
agent-mobile screenshot /tmp/<command>_evidence.png
# → 異常系も確認

# 7. コミット
git add .
git commit -m "feat: add <command> command"
```

## トラブルシューティング

### スクリプトが動作しない

**問題**: `./scripts/new-command.sh: Permission denied`

**解決**:
```bash
chmod +x .claude/skills/agent-mobile-dev/scripts/*.sh
```

### プロジェクトルートが見つからない

**問題**: `proto/idb.proto: No such file or directory`

**原因**: スクリプトをプロジェクトルート以外から実行している

**解決**: プロジェクトルートから実行してください:
```bash
cd /Users/r0227n/Dev/agent-mobile
./.claude/skills/agent-mobile-dev/scripts/platform-check.sh <feature>
```

### 実機確認でコマンドが見つからない

**問題**: `agent-mobile: command not found`

**原因**: バイナリがビルドされていない、またはPATHが通っていない

**解決**:
```bash
# ビルド確認
cargo build
ls -la target/debug/agent-mobile

# PATHに追加、または絶対パスで実行
./target/debug/agent-mobile <command> [args]
```

## 開発フローの原則

### 必須ステップ: 実機動作確認

**⚠️ 最重要**: ビルド成功≠正しい動作

実機確認は**必須ステップ**です。ビルドが通っても、実際の動作を確認するまでコミットしないでください。

**理由**:
1. **視覚的な検証**: UI操作は視覚的に確認しないと正しさが判断できない
2. **実環境での動作**: シミュレータと実機で挙動が異なる場合がある
3. **エラーメッセージ**: 実際のユーザーが見るエラーメッセージを確認
4. **パフォーマンス**: 実行速度、レスポンスが適切か
5. **統合動作**: 他の機能との統合動作が正しいか

### Progressive Disclosure

- **SKILL.mdは簡潔に**: コア開発フローと判断ツリーのみ（602行）
- **詳細はreferences/へ**: パターンの複数例、詳細説明、網羅的リスト（合計2,700行以上）
- **文脈連動リンク**: 各判断ポイントから適切なreferenceへ誘導

## 関連ドキュメント

**プロジェクト内:**
- `CLAUDE.md`: AI開発者向けクイックスタート
- `README.md`: ユーザー向け使用方法
- `docs/ARCHITECTURE.md`: アーキテクチャ概要
- `proto/idb.proto`: gRPC API定義
- `.claude/rules/cli-feature.md`: iOS実装判断基準（非推奨、このスキルに統合済み）

**このスキル内:**
- `SKILL.md`: メインスキル定義
- `references/environment-setup.md`: 環境セットアップ詳細
- `references/platform-decisions.md`: iOS実装判断基準詳細
- `references/implementation-patterns.md`: コーディングパターン集
- `references/testing-guide.md`: テスト戦略詳細
- `references/architecture.md`: アーキテクチャ詳細

## 貢献

スキルの改善提案、バグ報告はissueまたはプルリクエストでお願いします。

## ライセンス

agent-mobileプロジェクトと同じライセンスに従います。

---

**まとめ**: 新機能追加時は 環境確認（Phase 0） → `SKILL.md` → スクリプト → 実機確認（必須!） → コミット
