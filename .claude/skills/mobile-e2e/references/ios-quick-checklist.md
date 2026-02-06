# iOS E2E テスト軽量版チェックリスト

このチェックリストは `agent-mobile` CLI の iOS 向け主要機能を素早くテストするための軽量版です。
各カテゴリから代表的な項目を抽出しています。

## セッション管理 (最初に実行)

- [ ] session create --session e2e-test
- [ ] session list
- [ ] session show --session e2e-test

## デバイス確認

- [ ] device list -p ios
- [ ] device list --format json

## アプリ管理

- [ ] app launch com.apple.mobilesafari --session e2e-test
- [ ] app launch com.apple.mobilesafari --format json --session e2e-test
- [ ] app terminate com.apple.mobilesafari --session e2e-test

## スナップショット

- [ ] snapshot --session e2e-test
- [ ] snapshot --interactive --session e2e-test
- [ ] snapshot --compact --session e2e-test
- [ ] snapshot --format json --session e2e-test
- [ ] snapshot --depth 3 --session e2e-test

## 要素検索

- [ ] find type Button --first --session e2e-test
- [ ] find type TextField --first --session e2e-test
- [ ] find type Button --format json --session e2e-test
- [ ] find enabled --first --session e2e-test
- [ ] find label "Safari" --session e2e-test

## UI操作 - タップ

- [ ] tap @e1 --session e2e-test (snapshot で確認した要素)
- [ ] tap 200,400 --session e2e-test
- [ ] tap home --session e2e-test

## UI操作 - テキスト入力

- [ ] fill @eN "test text" --session e2e-test (TextField に対して)
- [ ] type " appended" --session e2e-test

## UI操作 - スワイプ/スクロール

- [ ] swipe up --session e2e-test
- [ ] swipe down --session e2e-test
- [ ] scroll up --session e2e-test
- [ ] scroll down --session e2e-test

## UI操作 - 長押し

- [ ] long-press @eN --session e2e-test
- [ ] long-press @eN --duration 2 --session e2e-test

## 要素情報取得

- [ ] get @eN --session e2e-test
- [ ] get text @eN --session e2e-test
- [ ] get box @eN --session e2e-test
- [ ] get text @eN --format json --session e2e-test

## 状態確認

- [ ] is visible @eN --session e2e-test
- [ ] is exists @eN --session e2e-test
- [ ] is enabled @eN --session e2e-test

## 待機

- [ ] wait idle --session e2e-test
- [ ] wait visible @eN --timeout 5s --session e2e-test

## スクリーンショット

- [ ] screenshot --session e2e-test
- [ ] screenshot -o ./test-screenshot.png --session e2e-test
- [ ] screenshot --format jpeg --session e2e-test

## 録画

- [ ] record --time-limit 3 --session e2e-test

## ログ

- [ ] console --session e2e-test (Ctrl+C で停止)

## クリップボード

- [ ] device pbcopy "test clipboard" --session e2e-test
- [ ] device pbpaste --session e2e-test

## グローバルオプション確認

- [ ] --help (任意のコマンドで)
- [ ] --version
- [ ] --format text (任意のコマンドで)
- [ ] --format json (任意のコマンドで)

## セッション終了

- [ ] session destroy --session e2e-test

## サマリー

**検証対象パターン総数**: 約50項目
- セッション管理: 4項目
- デバイス確認: 2項目
- アプリ管理: 3項目
- スナップショット: 5項目
- 要素検索: 5項目
- UI操作: 10項目
- 要素情報取得: 4項目
- 状態確認: 3項目
- 待機: 2項目
- スクリーンショット: 3項目
- 録画: 1項目
- ログ: 1項目
- クリップボード: 2項目
- グローバルオプション: 4項目
- セッション終了: 1項目
