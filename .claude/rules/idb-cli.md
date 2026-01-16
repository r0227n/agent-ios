---
paths:
  - "src/**/*.rs"
  - "src/cli/*.rs"
---

# src/cli/ ディレクトリルール

Python idb CLI を Rust で再実装するディレクトリ。`idb_companion` (Swift/ObjC) との gRPC 通信を行う。

## ファイル構造

```
src/cli/
├── mod.rs              # CLI パーサー (clap)
└── idb/
    ├── mod.rs          # IdbCommands enum（サブコマンド定義）
    └── <command>.rs    # 各コマンドの実装
```

## コマンド実装パターン

各コマンドは以下のパターンに従う:

```rust
pub async fn run(
    udid: Option<String>,
    // その他の引数
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. CompanionState から companion を取得
    let state = CompanionState::default();
    let companion = match udid.as_deref() {
        Some(u) => state.find_by_udid(u).ok_or_else(|| format!("No companion found for UDID: {}", u))?,
        None => state.get_companions().into_iter().next().ok_or("No companions available.")?,
    };

    // 2. gRPC 接続
    let address = companion.address().ok_or("Companion has no valid address")?;
    let mut client = match &address {
        Address::DomainSocket { path } => IdbClient::connect_uds(path).await?,
        Address::Tcp { host, port } => IdbClient::connect_tcp(host, *port).await?,
    };

    // 3. RPC 呼び出し
    client.your_method().await?;

    Ok(())
}
```

## 新しいコマンドの追加手順

1. `src/cli/idb/mod.rs` に `IdbCommands` enum variant 追加
2. `src/cli/idb/<command>.rs` にコマンド実装
3. `src/main.rs` にルーティング追加
4. `proto/idb.proto` に gRPC 定義追加（必要な場合）
5. `src/grpc/client.rs` に gRPC メソッド追加
6. `tests/<command>_integration.rs` に統合テスト追加

## テスト戦略

- **ユニットテスト**: ヘルパー関数を抽出して `#[cfg(test)] mod tests` で実装
- **統合テスト**: Python idb との出力を比較（`tests/` ディレクトリ）

## 参照先

- Python 実装: `idb/idb/cli/commands/`
- gRPC クライアント: `src/grpc/client.rs`
- 型定義: `src/types/`
