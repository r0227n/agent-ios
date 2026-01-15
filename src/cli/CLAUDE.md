# CLI 実装ガイド

Python idb CLI を Rust で再実装する際のガイド。

## 実装済みコマンド

```bash
# ターゲット一覧
agent-mobile idb list-targets [--only <device|simulator|mac>] [--human]

# アプリ起動
agent-mobile idb launch [OPTIONS] <BUNDLE_ID> [APP_ARGUMENTS]...
```

## 新しいコマンドの追加方法

1. `src/cli/idb/mod.rs` に `IdbCommands` enum variant 追加
2. `src/cli/idb/<command>.rs` にコマンド実装
3. `src/main.rs` にルーティング追加
4. `proto/idb.proto` に必要な gRPC 定義追加
5. `src/grpc/client.rs` に gRPC メソッド追加
6. `tests/<command>_integration.rs` に Python idb との互換性テスト追加

## 参照

Python 実装は `idb/idb/cli/commands/` を参照。
