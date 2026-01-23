#!/bin/bash
# ck JSONL 出力パーサー
# 使用例: ck --sem --jsonl "query" | ./ck-parse.sh
# オプション: -f (fields), -s (sort), --files-only

set -e

# デフォルト値
FIELDS=""
SORT_BY=""
FILES_ONLY=false

# ヘルプ表示
show_help() {
    cat << EOF
Usage: ck --sem --jsonl "query" | ck-parse.sh [OPTIONS]

ck の JSONL 出力を解析して整形

Options:
  --files-only    ファイルパスのみ出力
  -f FIELDS       表示するフィールド (カンマ区切り: path,score,lines)
  -s FIELD        ソートするフィールド (score, path)
  -h              ヘルプを表示

Examples:
  ck --sem --jsonl "error" | ./ck-parse.sh --files-only
  ck --sem --jsonl "auth" | ./ck-parse.sh -f path,score
  ck --sem --jsonl "grpc" | ./ck-parse.sh -s score
EOF
}

# オプション解析
while [ $# -gt 0 ]; do
    case $1 in
        --files-only)
            FILES_ONLY=true
            shift
            ;;
        -f)
            FIELDS="$2"
            shift 2
            ;;
        -s)
            SORT_BY="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            echo "Error: Unknown option '$1'" >&2
            show_help
            exit 1
            ;;
    esac
done

# jq がインストールされているか確認
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required but not installed" >&2
    echo "Install with: brew install jq" >&2
    exit 1
fi

# 入力を一時保存（複数回処理のため）
INPUT=$(cat)

if [ -z "$INPUT" ]; then
    echo "No results" >&2
    exit 0
fi

# ファイルパスのみの場合
if [ "$FILES_ONLY" = true ]; then
    echo "$INPUT" | jq -r '.path // empty' | sort -u
    exit 0
fi

# フィールド指定がある場合
if [ -n "$FIELDS" ]; then
    # カンマ区切りのフィールドを jq 形式に変換
    JQ_FIELDS=$(echo "$FIELDS" | sed 's/,/,./g' | sed 's/^/./')
    echo "$INPUT" | jq -r "[$JQ_FIELDS] | @tsv"
    exit 0
fi

# ソート指定がある場合
if [ -n "$SORT_BY" ]; then
    case $SORT_BY in
        score)
            echo "$INPUT" | jq -s 'sort_by(.score) | reverse | .[]' | jq -c '.'
            ;;
        path)
            echo "$INPUT" | jq -s 'sort_by(.path) | .[]' | jq -c '.'
            ;;
        *)
            echo "Error: Unknown sort field '$SORT_BY'" >&2
            exit 1
            ;;
    esac
    exit 0
fi

# デフォルト: 整形して出力
echo "$INPUT" | jq -r 'if .path then "\(.path):\(.span.line_start // "?"): \(.score // "-")" else empty end'
