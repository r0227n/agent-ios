#!/bin/bash
# ck セマンティック検索ラッパー
# 使用例: ./ck-search.sh "error handling" src/
# オプション: -t (threshold), -k (topk), -m (mode: sem/hybrid/lex)

set -e

# デフォルト値
MODE="sem"
TOPK=""
THRESHOLD=""
EXTRA_ARGS=""

# ヘルプ表示
show_help() {
    cat << EOF
Usage: ck-search.sh [OPTIONS] QUERY [PATH]

セマンティック検索を簡単に実行するラッパースクリプト

Options:
  -m MODE      検索モード: sem (default), hybrid, lex
  -k NUM       結果数制限 (topk)
  -t NUM       スコア閾値 (threshold)
  -h           ヘルプを表示

Examples:
  ./ck-search.sh "error handling"
  ./ck-search.sh -m hybrid "authentication logic" src/
  ./ck-search.sh -k 10 -t 0.5 "grpc client" .
EOF
}

# オプション解析
while getopts "m:k:t:h" opt; do
    case $opt in
        m) MODE="$OPTARG" ;;
        k) TOPK="$OPTARG" ;;
        t) THRESHOLD="$OPTARG" ;;
        h) show_help; exit 0 ;;
        *) show_help; exit 1 ;;
    esac
done
shift $((OPTIND-1))

# クエリ必須チェック
if [ -z "$1" ]; then
    echo "Error: Query is required" >&2
    show_help
    exit 1
fi

QUERY="$1"
PATH_ARG="${2:-.}"

# モードに応じたフラグを設定
case $MODE in
    sem)
        MODE_FLAG="--sem"
        ;;
    hybrid)
        MODE_FLAG="--hybrid"
        ;;
    lex)
        MODE_FLAG=""
        ;;
    *)
        echo "Error: Unknown mode '$MODE'. Use: sem, hybrid, or lex" >&2
        exit 1
        ;;
esac

# オプション構築
if [ -n "$TOPK" ]; then
    EXTRA_ARGS="$EXTRA_ARGS --topk $TOPK"
fi

if [ -n "$THRESHOLD" ]; then
    EXTRA_ARGS="$EXTRA_ARGS --threshold $THRESHOLD"
fi

# ck コマンド実行
exec ck $MODE_FLAG --jsonl --no-snippet $EXTRA_ARGS "$QUERY" "$PATH_ARG"
