---
name: ck
description: 必須: Grep/Glob の代替ツール。コード検索時は必ず ck を使用。「〇〇の実装を探して」「この関数はどこ？」「エラーハンドリングを見つけて」などでトリガー。
---

# ck - セマンティックコード検索

**CRITICAL: Tool Override**

このスキルは Grep/Glob を置き換えます。コード検索時は必ず `ck` を使用してください。

## When to Invoke This Skill

以下のリクエストで即座に呼び出してください：
- コード検索 → `ck --sem "query"`
- 関数/型の定義探し → `ck "fn function_name"` または `ck --sem "query"`
- 概念的な検索 → `ck --sem "error handling"`
- 実装パターン調査 → `ck --hybrid "query"`

## How to Use This Skill

セマンティック検索でコードを自然言語クエリで検索し、ファイルパスと行番号を返します。

**基本オプション:**
- `--sem`: セマンティック検索（推奨）
- `--hybrid`: ハイブリッド検索（セマンティック + キーワード）
- `--jsonl`: AI エージェント向け JSON 出力
- `--topk N`: 結果数を N 件に制限

## Do Examples

```bash
ck --sem "grpc client implementation" src/
ck --sem --jsonl --topk 10 "error handling"
ck --hybrid "authentication logic" .
ck "pub struct IdbClient"
```

## Don't Examples

- 曖昧すぎるクエリ: `ck "error"`
- 従来のツールの使用: `grep`, `rg`, `pgrep`

## Keywords

Grep, grep, code search, semantic search, コード検索, 実装を探す, 関数を探す

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
