---
name: ck
description: セマンティック検索ツール。コード検索時に Grep ではなく ck を優先使用。「〇〇の実装を探して」「この関数はどこ？」「エラーハンドリングを見つけて」などのコード検索タスクでトリガー。
---

# ck - セマンティックコード検索

## Overview

`ck` は高速なセマンティック検索ツールです。Grep の代わりに使用することで、より意味的に正確なコード検索が可能になります。

## 基本的な使い方

### セマンティック検索（推奨）
概念的な検索に最適。「エラーハンドリング」「認証ロジック」など。

```bash
ck --sem "検索クエリ" [パス]
```

### ハイブリッド検索
セマンティックとキーワードの両方の利点が必要な場合。

```bash
ck --hybrid "検索クエリ" [パス]
```

### 正規表現検索（デフォルト）
完全一致が必要な場合。

```bash
ck "パターン" [パス]
```

## 推奨オプション

| オプション | 説明 |
|-----------|------|
| `--jsonl` | AI エージェント向け JSON Lines 出力 |
| `--no-snippet` | スニペット省略（高速化） |
| `--topk N` | 結果数を N 件に制限 |
| `--threshold` | スコア閾値（低スコアの結果を除外） |

### 例

```bash
# セマンティック検索、JSON出力、上位10件
ck --sem --jsonl --topk 10 "grpc client implementation" src/

# ハイブリッド検索、閾値0.5以上
ck --hybrid --threshold 0.5 "error handling" .
```

## 使い分けガイド

| 検索タイプ | 使用するモード | 例 |
|-----------|--------------|-----|
| 概念的な検索 | `--sem` | 「エラーハンドリング」「認証ロジック」 |
| 両方の利点 | `--hybrid` | 「gRPC クライアント」 |
| 完全一致 | （フラグなし） | `fn get_client` |

## ラッパースクリプト

### ck-search.sh
セマンティック検索を簡単に実行。

```bash
./scripts/ck-search.sh "error handling" src/
./scripts/ck-search.sh -m hybrid -k 10 "authentication" .
```

### ck-index.sh
インデックス管理。

```bash
./scripts/ck-index.sh status    # 状態確認
./scripts/ck-index.sh build     # インデックス作成
./scripts/ck-index.sh rebuild   # 再構築
```

### ck-parse.sh
JSONL 出力の解析。

```bash
ck --sem --jsonl "query" | ./scripts/ck-parse.sh --files-only
ck --sem --jsonl "query" | ./scripts/ck-parse.sh -f path,score
```

## Resources

### scripts/
- `ck-search.sh` - セマンティック検索ラッパー
- `ck-index.sh` - インデックス管理
- `ck-parse.sh` - JSONL 出力パーサー

### references/
- `options.md` - 詳細オプションリファレンス
