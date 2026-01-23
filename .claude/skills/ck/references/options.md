# ck 詳細オプションリファレンス

## 検索モードオプション

| オプション | 説明 |
|-----------|------|
| `--sem` | セマンティック検索モード。埋め込みベクトルを使用した意味的検索 |
| `--hybrid` | ハイブリッド検索モード。セマンティックとキーワードの組み合わせ |
| （なし） | 正規表現検索モード（デフォルト） |

## 出力オプション

| オプション | 説明 | デフォルト |
|-----------|------|----------|
| `--jsonl` | JSON Lines 形式で出力 | off |
| `--no-snippet` | コードスニペットを省略 | off |
| `--color` | カラー出力を強制 | auto |
| `--no-color` | カラー出力を無効 | off |

## フィルタリングオプション

| オプション | 説明 | 例 |
|-----------|------|-----|
| `--topk N` | 結果数を N 件に制限 | `--topk 10` |
| `--threshold F` | スコア閾値（0.0-1.0） | `--threshold 0.5` |
| `--type T` | ファイルタイプでフィルタ | `--type rust` |
| `--glob G` | グロブパターンでフィルタ | `--glob "*.rs"` |

## インデックス管理コマンド

### インデックス作成
```bash
ck index build [PATH]
```

### インデックス状態確認
```bash
ck index status [PATH]
```

### インデックス削除
```bash
ck index clean [PATH]
```

## JSONL 出力フォーマット

`--jsonl` オプション使用時の出力フォーマット:

```json
{
  "path": "src/main.rs",
  "span": {
    "byte_start": 100,
    "byte_end": 200,
    "line_start": 42,
    "line_end": 50
  },
  "language": "rust",
  "score": 0.85
}
```

### フィールド説明

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `path` | string | ファイルパス |
| `span.byte_start` | number | 開始バイト位置 |
| `span.byte_end` | number | 終了バイト位置 |
| `span.line_start` | number | 開始行番号 |
| `span.line_end` | number | 終了行番号 |
| `language` | string | 検出された言語 |
| `score` | number | マッチスコア（0.0-1.0） |

## 高度な使用例

### AI エージェント向けの最適な設定
```bash
ck --sem --jsonl --no-snippet --topk 20 "検索クエリ" .
```

### 特定のファイルタイプのみ検索
```bash
ck --sem --type rust "error handling" src/
```

### グロブパターンでフィルタ
```bash
ck --sem --glob "**/*.rs" "grpc client" .
```

### パイプラインでの使用
```bash
# ファイルパスのみ抽出
ck --sem --jsonl "query" | jq -r '.path' | sort -u

# スコア順にソート
ck --sem --jsonl "query" | jq -s 'sort_by(.score) | reverse | .[]'
```

## トラブルシューティング

### インデックスが古い場合
```bash
ck index rebuild .
```

### 検索結果が少ない場合
- `--threshold` を下げる
- `--topk` を増やす
- `--hybrid` モードを試す

### パフォーマンスが遅い場合
- `--no-snippet` を使用
- 検索パスを絞る
- インデックスを再構築
