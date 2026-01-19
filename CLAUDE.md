 ## 技術スタック

 - **Rust 2021**: メインアプリケーション
 - **tokio 1.49**: 非同期ランタイム
 - **tonic 0.12 + prost 0.13**: gRPC クライアント/Protocol Buffers
 - **clap 4.5**: CLI フレームワーク (derive)
 - **idb_companion**: Swift/ObjC ダエモン（外部プロセス）
 - **Python idb**: 参照実装（サブモジュール）

 ## 開発コマンド

 ### アーキテクチャ
```txt
 レイヤー構造とデータフロー

 ┌─────────────────────────────────────────────────────────────┐
 │  CLI Layer (src/cli/)                                        │
 │  - コマンド引数解析 (clap)                                  │
 │  - UDID 指定 → CompanionResolver へ                         │
 │  - 出力フォーマット (JSON/Human)                            │
 └─────────────────────────────────────────────────────────────┘
               ↓ (udid: Option<&str>)
 ┌─────────────────────────────────────────────────────────────┐
 │  Companion Resolution (src/platform/ios/companion/)          │
 │  ┌─────────────────────────────────────────────────────────┐│
 │  │ 1. CompanionState                                       ││
 │  │    - /tmp/idb/state から companion 情報読み込み        ││
 │  │    - JSON: [{udid, path/host/port, pid}]               ││
 │  │ 2. CompanionResolver                                    ││
 │  │    - UDID 指定時: state から該当 companion 検索        ││
 │  │    - UDID 未指定: 単一 companion を自動選択            ││
 │  │    - companion 不在: CompanionSpawner で自動起動       ││
 │  │ 3. 解決結果: ResolvedCompanion {address, udid}         ││
 │  └─────────────────────────────────────────────────────────┘│
 └─────────────────────────────────────────────────────────────┘
               ↓ (Address: DomainSocket | TCP)
 ┌─────────────────────────────────────────────────────────────┐
 │  gRPC Client (src/platform/ios/grpc/client.rs)              │
 │  ┌─────────────────────────────────────────────────────────┐│
 │  │ IdbClient                                               ││
 │  │ - connect_uds(path): Unix Domain Socket 接続           ││
 │  │ - connect_tcp(host, port): TCP 接続                    ││
 │  │ - CompanionServiceClient<Channel> をラップ            ││
 │  └─────────────────────────────────────────────────────────┘│
 │                                                              │
 │  カテゴリ別メソッド (src/platform/ios/grpc/):               │
 │  - app.rs: launch, terminate, list_apps                     │
 │  - file.rs: ls, mkdir, push, pull                           │
 │  - hid.rs: tap, swipe, button, text                         │
 │  - target.rs: describe, list                                │
 │  - test.rs: xctest_run, xctest_list                         │
 └─────────────────────────────────────────────────────────────┘
               ↓ (gRPC Request/Response または Stream)
 ┌─────────────────────────────────────────────────────────────┐
 │  Proto Layer (tonic + prost)                                 │
 │  - build.rs で proto/idb.proto からコード生成              │
 │  - Unix Domain Socket トランスポート (tower service_fn)    │
 │  - ストリーミング RPC 対応                                  │
 └─────────────────────────────────────────────────────────────┘
               ↓ (Unix Domain Socket / TCP)
 ┌─────────────────────────────────────────────────────────────┐
 │  idb_companion (Swift/ObjC 外部プロセス)                    │
 │  - iOS Simulator/Device との通信                            │
 │  - TCC データベース操作 (パーミッション)                   │
 │  - Framebuffer 読み取り (スクリーンショット)               │
 └─────────────────────────────────────────────────────────────┘
 ```
 