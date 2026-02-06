# agent-mobile-core Architecture

`agent-mobile-core` は workspace 全体で共有する platform 非依存の基本型を集約する crate です。CLI の実装詳細や transport には触れず、「他の crate が共通で参照できる最小の土台」を提供します。

## 1. 目的

- iOS / Android をまたぐ enum と型を共通化する
- snapshot の基本表現を backend から独立させる
- trait と error を用意し、将来的な抽象化の境界を維持する
- file / stdout 出力を `OutputWriter` として共有する

## 2. 公開 API / 主要型

`src/lib.rs` は次を再 export します。

- `error::{Error, Result}`
- `io::OutputWriter`
- `snapshot::{Frame, RawElement, extract_traits_for_type, is_interactive_type}`
- `types::{Platform, ScrollDirection, DeviceInfo, TargetType, InstalledArtifact, ...}`

特に重要な型は次です。

| 型 | 用途 |
| --- | --- |
| `Platform` | `ios` / `android` を表す共通 enum |
| `ScrollDirection` | swipe / scroll 方向の共通 enum |
| `Frame` | element の screen 座標 |
| `RawElement` | backend 抽出直後の中間 snapshot 表現 |
| `OutputWriter` | stdout / file / tee 出力 |
| `DeviceOperations`, `AppOperations`, `FileOperations` | 抽象化 trait |

## 3. 内部モジュール責務

### `error`

- crate 横断で使える軽量 error 型
- `String` や `&str` からの変換を持つ

### `types`

- `Platform`, `ScrollDirection` のような CLI と backend 双方で使う enum
- target / install / output に関する shared type
- `clap` feature 有効時だけ `ValueEnum` を derive する

### `snapshot`

- `Frame` と `RawElement` を定義する
- `extract_traits_for_type()` が element type から trait を推定する
- `is_interactive_type()` が iOS / Android の interactive element 判定をまとめる

### `io`

- `OutputWriter` が stdout / file / tee を隠蔽する
- `console` や将来の streaming command から直接使える

### `traits`

- `DeviceOperations`
- `AppOperations`
- `FileOperations`
- `MobileDevice`

現状の CLI 実装はこれらを全面採用していませんが、抽象化の受け皿として維持されています。

## 4. 他 crate との依存関係

`agent-mobile-core` は workspace 内で最も下位の層です。

- 依存先: `serde`, `thiserror`, `async-trait`, optional `clap`
- 依存元: ルート CLI、`gateway`、`platform-ios`、`platform-android`

この crate 自体は platform backend を知らず、逆方向依存を持ちません。

## 5. 代表フロー

### Snapshot 抽出の共通土台

1. backend が platform 固有データを取得する
2. backend が `RawElement` tree に変換する
3. CLI 側 `src/snapshot/` が `RawElement` を `SnapshotElement` と `@eN` ref に変換する

### 出力

1. command が `OutputWriter` を生成する
2. stdout / file / tee の違いを caller から隠したまま write / flush する

## 6. 制約と設計判断

- backend 事情を持ち込まないことを優先し、ここで transport 依存型は定義しない
- trait は将来の抽象化余地として残しているが、現状の command は platform crate を直接呼ぶことも多い
- snapshot の共通化は `RawElement` までに留め、`@eN` ref や cache は CLI crate 側の責務にしている
