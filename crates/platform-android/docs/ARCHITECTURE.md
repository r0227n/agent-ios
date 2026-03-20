# agent-mobile-platform-android Architecture

`agent-mobile-platform-android` は Android backend です。中心は ADB server (`127.0.0.1:5037`) との直接通信で、UI dump、input、permission、logcat をそれぞれ専用モジュールに分けています。

## 1. 目的

- ADB native protocol を使って Android device/emulator を操作する
- UIAutomator dump を snapshot 系 command の入力に変換する
- app lifecycle、permission、screenshot、screenrecord 周辺機能を提供する

## 2. 公開 API / 主要型

`src/lib.rs` が再 export する主な API は次です。

| API | 役割 |
| --- | --- |
| `AdbConnection` | ADB native connection の基盤 |
| `list_devices()`, `list_avds()` | device/emulator discovery |
| `LogcatStream` | Android log streaming |
| `dump_ui()`, `parse_ui_hierarchy()` | UIAutomator dump と parse |
| `screenshot()`, `screenshot_bytes()` | screenshot 取得 |
| `grant_permission()` など | runtime permission 操作 |
| `extract_android_elements()` | `AccessibilityElement` を `RawElement` へ変換 |

## 3. 内部モジュール責務

### `adb/connection.rs`

- `adb_client` を使い ADB server に接続する
- shell command、pull/push、install/uninstall、framebuffer 取得をまとめる
- `list_devices()` や `list_avds()` の基盤でもある

### `adb/commands.rs`

- ADB availability、device property (`getprop`) 取得、device 種別判定を提供する
- 高頻度の情報取得系 helper を分離している

### `adb/input.rs`

- `input tap`, `input swipe`, `input text`, `input keyevent`
- long press は同一点 swipe で表現する
- screen size 取得もここで扱う

### `adb/app.rs`

- `monkey` や `am start` による app launch
- `pm list packages`, `dumpsys package` による app 情報取得
- install / uninstall / clear data

### `adb/permission.rs`

- `pm grant`, `pm revoke`, `pm reset-permissions`
- `dumpsys package` から runtime permission 状態を parse

### `adb/uiautomator.rs`

- `uiautomator dump /sdcard/window_dump.xml`
- `cat /sdcard/window_dump.xml`
- XML を `AccessibilityElement` 列へ変換
- `find_by_text`, `find_by_id`, `find_by_type` の補助関数を持つ

### `adb/logcat.rs`

- ADB wire protocol で `host:transport:*` と `shell:logcat` を送る
- `TcpStream` を `BufReader` で包み、line 単位の stream にする

### `snapshot`

- Android 固有 element を `agent-mobile-core::snapshot::RawElement` に揃える
- clickable / scrollable などの属性を共通 trait に補完する

## 4. 他 crate との依存関係

- 下位依存: `agent-mobile-core`, `adb_client`, `tokio`, `serde`
- 主な依存元: ルート CLI、`agent-mobile-gateway`
- 外部依存: 稼働中の ADB server

## 5. 代表フロー

### Tap

1. CLI が target serial を解決する
2. snapshot または locator 解決で座標を確定する
3. `adb::input::tap()` が `AdbConnection` で `input tap x y` を実行する

### Snapshot

1. `dump_ui()` が UIAutomator dump を作る
2. XML を `AccessibilityElement` に parse する
3. `extract_android_elements()` が `RawElement` へ変換する
4. CLI 側で `SnapshotElement` と `@eN` ref を生成する

### Console

1. `LogcatStream::open()` が TCP `:5037` に接続する
2. `host:transport:<serial>` または `host:transport-any` を送る
3. `shell:logcat` を開始し、caller が `next_line()` で読み出す

## 6. 制約と設計判断

- Android 側は追加 daemon を起動せず、既存の ADB server を前提にする
- `adb` CLI ではなく native protocol を優先するが、実際の shell command 自体は device 上で多用する
- UIAutomator dump は iOS の accessibility API より情報量と精度が異なるため、snapshot の品質差を前提に設計している
- XML parser は uiautomator 出力を前提にした簡易実装であり、一般 XML parser ではない
