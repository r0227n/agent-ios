# agent-mobile-gateway Architecture

`agent-mobile-gateway` は workspace の中で薄い facade として機能する crate です。大規模な domain layer ではなく、CLI 層から切り出す価値がある共通ロジックだけを持ちます。

## 1. 目的

- iOS / Android の platform detection を再利用可能な形でまとめる
- console streaming の platform 差分を単一点で吸収する
- Android 向けの最小限の高レベル API を提供する

## 2. 公開 API / 主要型

`src/lib.rs` から公開している主な API は次です。

| API | 役割 |
| --- | --- |
| `DeviceResolver` | booted simulator / connected Android device を見て platform を推定する |
| `stream_console_logs()` | iOS / Android の console streaming を切り替える |
| `AndroidDevice` | Android log streaming 向けの軽量 facade |

## 3. 内部モジュール責務

### `platform`

- `parse_platform()` で文字列を `Platform` に変換する
- `detect_platform()` で iOS 優先、その次に Android を確認する
- `has_booted_ios_simulators()` は `platform-ios::simctl::list_simulators()`
- `has_android_devices()` は `platform-android::list_devices()`

### `console`

- `stream_console_logs()` が public entrypoint
- iOS は `xcrun simctl spawn <udid> log stream --style compact`
- Android は `AndroidDevice::stream_logs()` 経由で `LogcatStream`
- どちらも `watch::Receiver<bool>` で stop を受け取る

### `api/android`

- `AndroidDevice::connect()` で ADB server 到達性を確認する
- `stream_logs()` だけを提供する小さな wrapper
- 現状は Android 全体 abstraction ではなく、console 用 API が中心

## 4. 他 crate との依存関係

- 依存先: `agent-mobile-core`, `agent-mobile-platform-ios`, `agent-mobile-platform-android`
- 主な依存元: ルート CLI (`src/device.rs`, `src/console.rs`, `src/core/*.rs`)

依存の向きとしては、CLI の直下にある再利用層であり、backend を横断して呼ぶ責務だけを持ちます。

## 5. 代表フロー

### Platform detection

1. caller が `DeviceResolver::detect_platform()` を呼ぶ
2. booted iOS simulator があれば `Platform::Ios`
3. なければ Android device の存在を確認して `Platform::Android`
4. どちらも無ければ error

### Console streaming

1. CLI が `OutputWriter` と stop channel を構築する
2. `stream_console_logs()` が platform ごとに分岐する
3. iOS は child process、Android は TCP logcat stream を開始する
4. line 単位で writer に書き込み、stop 受信で終了する

## 6. 制約と設計判断

- `gateway` は厚い abstraction layer を目指していない
- ルート CLI から直接 platform crate を呼ぶ実装も残している
- 現状の価値は「横断的だが小さい機能」を局所的に切り出す点にある
- Android 向け API は `console` に必要な最小範囲に留まり、iOS と対称な device facade にはなっていない
