#!/bin/bash
# ck インデックス管理スクリプト
# 使用例: ./ck-index.sh status  # 状態確認
#         ./ck-index.sh build   # インデックス作成
#         ./ck-index.sh clean   # クリーンアップ

set -e

# ヘルプ表示
show_help() {
    cat << EOF
Usage: ck-index.sh COMMAND [OPTIONS]

インデックスの作成/更新/状態確認/クリーンアップ

Commands:
  status    インデックス状態を表示
  build     インデックスを作成/更新
  clean     不要なインデックスを削除
  rebuild   完全に再構築

Options:
  -p PATH   対象パス (default: .)
  -h        ヘルプを表示

Examples:
  ./ck-index.sh status
  ./ck-index.sh build -p src/
  ./ck-index.sh rebuild
EOF
}

# デフォルト値
PATH_ARG="."

# -h フラグを先にチェック
for arg in "$@"; do
    if [ "$arg" = "-h" ] || [ "$arg" = "--help" ]; then
        show_help
        exit 0
    fi
done

# コマンド取得
COMMAND="${1:-}"
shift 2>/dev/null || true

# オプション解析
while getopts "p:" opt; do
    case $opt in
        p) PATH_ARG="$OPTARG" ;;
        *) show_help; exit 1 ;;
    esac
done

case $COMMAND in
    status)
        echo "=== ck Index Status ==="
        ck index status "$PATH_ARG" 2>/dev/null || echo "No index found for $PATH_ARG"
        ;;
    build)
        echo "=== Building Index ==="
        ck index build "$PATH_ARG"
        echo "Index built successfully"
        ;;
    clean)
        echo "=== Cleaning Index ==="
        ck index clean "$PATH_ARG" 2>/dev/null || echo "Nothing to clean"
        echo "Cleanup complete"
        ;;
    rebuild)
        echo "=== Rebuilding Index ==="
        ck index clean "$PATH_ARG" 2>/dev/null || true
        ck index build "$PATH_ARG"
        echo "Index rebuilt successfully"
        ;;
    "")
        echo "Error: Command is required" >&2
        show_help
        exit 1
        ;;
    *)
        echo "Error: Unknown command '$COMMAND'" >&2
        show_help
        exit 1
        ;;
esac
