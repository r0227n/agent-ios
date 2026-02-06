# agent-mobile-platform-ios Architecture

`agent-mobile-platform-ios` は iOS Simulator 向け backend です。単一 transport ではなく、`simctl` / CoreSimulator と XCUITest Runner HTTP API を組み合わせる hybrid 構成を採ります。

## 1. 目的

- simulator の lifecycle と app 管理を host 側から実行する
- XCUITest Runner を起動・再利用し、iOS UI の観測と操作を実現する
- iOS 固有データを CLI 共通の snapshot 表現へ変換する

## 2. 公開 API / 主要型

`src/lib.rs` からの主な公開面は次です。

| API | 役割 |
| --- | --- |
| `simctl::*` | boot / shutdown / install / uninstall / list / privacy |
| `coresim::*` | booted simulator 上の app launch / terminate 補助 |
| `XCUITestClient` | runner HTTP client |
| `ensure_runner_started()` | runner の起動と readiness 保証 |
| `extract_ios_elements()` | legacy accessibility JSON を `RawElement` へ変換 |

## 3. 内部モジュール責務

### `simctl`

- `management.rs` が `xcrun simctl` の command wrapper
- `list_simulators()` が JSON を parse し `DeviceInfo` を返す
- `cache.rs` が simulator 一覧を TTL 付きで cache する
- privacy grant / revoke / reset もここで扱う

### `coresim`

- `objc2` を使って CoreSimulator framework を呼ぶ
- booted simulator の app install / launch / terminate を補助する
- `simctl` が不得意な実行経路を host 側から補う

### `xcuitest/client.rs`

- `reqwest` ベースの HTTP client
- `/health`, `/ready`, `/snapshot`, `/query/first`, `/tap`, `/type`, `/clipboard/*` などを呼ぶ
- 返却 JSON を Rust 側の型へ deserialize する

### `xcuitest/runner.rs`

- XCUITest Runner build products の探索
- 必要なら `xcodebuild build-for-testing`
- `xcodebuild test-without-building` で `testStartAutomationServer` を起動
- `/ready` を poll して runner reuse / cleanup / restart を判断する

### `snapshot`

- legacy nested accessibility JSON を `RawElement` tree に変換する
- 現在の CLI fast path は runner `/snapshot` を優先するが、抽出器は互換パスとして残る

## 4. 他 crate との依存関係

- 下位依存: `agent-mobile-core`, `reqwest`, `tokio`, `objc2`, `plist`
- 主な依存元: ルート CLI、`agent-mobile-gateway`
- 外部依存: Xcode Command Line Tools, `xcrun simctl`, CoreSimulator framework, `xcodebuild`

## 5. 代表フロー

### UI command 実行

1. CLI が `prepare_xcuitest_with_policy()` を呼ぶ
2. その中で `ensure_runner_started()` が対象 UDID の runner を保証する
3. `XCUITestClient::ready_status()` を見て active app context を確認する
4. 必要なら `set_app()` で last active app を復元する
5. command が `/query/*`, `/snapshot`, `/tap`, `/type` などを呼ぶ

### App launch

1. CLI が target UDID を解決する
2. host 側で app launch を実行する
3. last active app を session/app-context に保存する
4. runner 側にも `set_app()` を送り、以降の accessibility context を揃える

### Device list

1. `simctl list devices --json` 相当を取得する
2. runtime ID から `iOS 17.0` のような version を復元する
3. `DeviceInfo` に変換し cache へ保存する

## 6. 制約と設計判断

- iOS の UI 自動化は XCUITest API 制約に従うため、Rust 単独では完結しない
- 端末管理と UI 自動化を別 transport に分け、各操作に適した経路を使う
- runner の reuse を優先し、毎回 build / 起動しない
- runner 起動後も active app context は別管理が必要なので、CLI 側に復元ロジックを持つ
