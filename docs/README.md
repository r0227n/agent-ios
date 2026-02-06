# agent-mobile ドキュメント

> agent-mobile の技術ドキュメント集

## 📚 ドキュメント一覧

### プラットフォーム別ガイド

#### Android

- **[android-console.md](./android-console.md)** - Android デバイスのリアルタイムログストリーミング
  - ADB サーバーのセットアップ
  - logcat ストリーミングの実装詳細
  - トラブルシューティング

### 今後追加予定

- **iOS Console** - iOS シミュレーターのログストリーミング
- **Architecture Overview** - 全体アーキテクチャ解説
- **API Reference** - Rust API ドキュメント
- **E2E Testing Guide** - 統合テストガイド

---

## クイックリンク

### 主要ドキュメント

- [README.md](../README.md) - プロジェクト概要 (English)
- [CLAUDE.md](../CLAUDE.md) - 開発ワークフロー (日本語)

### コードベース

- [crates/](../crates/) - Rust クレート
  - `core/` - プラットフォーム非依存のコア機能
  - `platform-ios/` - iOS 実装
  - `platform-android/` - Android 実装
  - `gateway/` - 高レベル API

---

## ドキュメント貢献ガイドライン

### 新規ドキュメントの作成

1. **適切なファイル名**: `platform-feature.md` (例: `ios-screenshot.md`)
2. **構成**:
   - 概要
   - 前提条件
   - 基本的な使い方
   - 実装詳細 (任意)
   - トラブルシューティング
   - 参考資料
3. **コード例**: 実際に動作する例を含める
4. **更新日**: ドキュメント末尾に記載

### スタイルガイド

- **日本語**: 技術ドキュメントは日本語で記述
- **コード**: コメントは英語、説明文は日本語
- **Markdown**: GitHub Flavored Markdown (GFM) を使用
- **絵文字**: セクション見出しのみ使用可 (本文では控える)

---

**最終更新**: 2026-02-07
