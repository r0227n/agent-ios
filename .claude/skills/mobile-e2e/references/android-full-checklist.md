# Android E2E テスト完全版チェックリスト

このチェックリストは `agent-mobile` CLI の Android 向け全機能をテストするための完全版です。

## UI操作関連コマンド

### tap
- [ ] tap @eN（参照でタップ）
- [ ] tap "text"（テキストでタップ）
- [ ] tap x,y（座標でタップ）
- [ ] tap home（ホームキーでタップ）
- [ ] tap back（バックキーでタップ）
- [ ] tap enter（エンターキーでタップ）
- [ ] tap @eN --session <SESSION>
- [ ] tap @eN --session <SESSION>
- [ ] tap @eN -u <SERIAL>

### check
- [ ] check @eN（参照でチェック）
- [ ] check "text"（テキストでチェック）
- [ ] check @eN --session <SESSION>
- [ ] check @eN -u <SERIAL>

### uncheck
- [ ] uncheck @eN（参照でアンチェック）
- [ ] uncheck "text"（テキストでアンチェック）
- [ ] uncheck @eN --session <SESSION>
- [ ] uncheck @eN -u <SERIAL>

### select
- [ ] select @eN "value"（参照でピッカーから選択）
- [ ] select "text" "value"（テキストでピッカーから選択）
- [ ] select @eN "value" --max-swipes 5
- [ ] select @eN "value" --max-swipes 10（デフォルト）
- [ ] select @eN "value" --max-swipes 20
- [ ] select @eN "value" --session <SESSION>
- [ ] select @eN "value" -u <SERIAL>

### long-press
- [ ] long-press @eN（参照で長押し）
- [ ] long-press "text"（テキストで長押し）
- [ ] long-press x,y（座標で長押し）
- [ ] long-press center（中央で長押し）
- [ ] long-press @eN --duration 1
- [ ] long-press @eN --duration 2
- [ ] long-press @eN --session <SESSION>
- [ ] long-press @eN -u <SERIAL>

### fill
- [ ] fill @eN "text"（参照でテキストを入力）
- [ ] fill "placeholder" "text"（プレースホルダでテキストを入力）
- [ ] fill @eN "text" --session <SESSION>
- [ ] fill @eN "text" -u <SERIAL>

### type
- [ ] type "text"（フォーカス済みフィールドにテキストを追記）
- [ ] type "text" --session <SESSION>
- [ ] type "text" -u <SERIAL>

### swipe
- [ ] swipe up（上方向にスワイプ）
- [ ] swipe down（下方向にスワイプ）
- [ ] swipe left（左方向にスワイプ）
- [ ] swipe right（右方向にスワイプ）
- [ ] swipe x1,y1,x2,y2（座標指定でスワイプ）
- [ ] swipe up --from @eN（要素から上方向にスワイプ）
- [ ] swipe up --from "text"
- [ ] swipe up --distance 100
- [ ] swipe up --distance 300
- [ ] swipe up --duration 0.5
- [ ] swipe up --duration 1.0
- [ ] swipe up --from @eN --distance 100 --duration 0.5
- [ ] swipe up --session <SESSION>
- [ ] swipe up -u <SERIAL>

### scroll
- [ ] scroll up（上方向にスクロール）
- [ ] scroll down（下方向にスクロール）
- [ ] scroll left（左方向にスクロール）
- [ ] scroll right（右方向にスクロール）
- [ ] scroll up --in @eN（特定要素内でスクロール）
- [ ] scroll up --in "text"
- [ ] scroll up --distance 100
- [ ] scroll up --distance 300
- [ ] scroll up --duration 0.5
- [ ] scroll up --duration 1.0
- [ ] scroll up --in @eN --distance 100 --duration 0.5
- [ ] scroll up --session <SESSION>
- [ ] scroll up -u <SERIAL>

## 要素情報取得関連コマンド

### get
- [ ] get text @eN（テキストを取得）
- [ ] get value @eN（値を取得）
- [ ] get attr @eN attribute_name（属性を取得）
- [ ] get box @eN（位置とサイズを取得）
- [ ] get count "text"（マッチ数を取得）
- [ ] get @eN（全プロパティを取得）
- [ ] get text @eN --format text
- [ ] get text @eN --format json
- [ ] get text @eN --session <SESSION>
- [ ] get text @eN -u <SERIAL>

### is
- [ ] is visible @eN（可視性を確認）
- [ ] is exists @eN（存在確認）
- [ ] is enabled @eN（有効確認）
- [ ] is disabled @eN（無効確認）
- [ ] is interactive @eN（インタラクティブ確認）
- [ ] is checked @eN（チェック状態確認）
- [ ] is visible "text"
- [ ] is visible @eN --session <SESSION>
- [ ] is visible @eN -u <SERIAL>

### find type
- [ ] find type Button（ボタン型を検索）
- [ ] find type EditText（テキストフィールド型を検索）
- [ ] find type Switch
- [ ] find type TextView
- [ ] find type ImageView
- [ ] find type Button --first（最初の要素のみ）
- [ ] find type Button --last（最後の要素のみ）
- [ ] find type Button --nth 0（N番目の要素）
- [ ] find type Button --all（すべて取得）
- [ ] find type Button --format text
- [ ] find type Button --format json
- [ ] find type Button tap（検索してタップ）
- [ ] find type Button long-press（検索して長押し）
- [ ] find type Button --nth 1 tap
- [ ] find type Button --session <SESSION>
- [ ] find type Button -u <SERIAL>

### find text
- [ ] find text "search term"（テキストで検索）
- [ ] find text "text" --first
- [ ] find text "text" --last
- [ ] find text "text" --nth 1
- [ ] find text "text" --all
- [ ] find text "text" --exact（完全一致検索）
- [ ] find text "text" --format text
- [ ] find text "text" --format json
- [ ] find text "Login" tap（検索してタップ）
- [ ] find text "Submit" long-press（検索して長押し）
- [ ] find text "Login" --exact tap
- [ ] find text "text" --session <SESSION>
- [ ] find text "text" -u <SERIAL>

### find label
- [ ] find label "label text"（ラベルで検索）
- [ ] find label "label" --first
- [ ] find label "label" --last
- [ ] find label "label" --nth 0
- [ ] find label "label" --all
- [ ] find label "label" --exact（完全一致検索）
- [ ] find label "label" --format text
- [ ] find label "label" --format json
- [ ] find label "Email" fill "test@example.com"（検索して入力）
- [ ] find label "Email" clear（検索してクリア）
- [ ] find label "Email" --exact fill "user@example.com"
- [ ] find label "label" --session <SESSION>
- [ ] find label "label" -u <SERIAL>

### find placeholder
- [ ] find placeholder "placeholder text"（プレースホルダで検索）
- [ ] find placeholder "text" --first
- [ ] find placeholder "text" --last
- [ ] find placeholder "text" --nth 0
- [ ] find placeholder "text" --all
- [ ] find placeholder "text" --exact（完全一致検索）
- [ ] find placeholder "text" --format text
- [ ] find placeholder "text" --format json
- [ ] find placeholder "Search..." fill "query"（検索して入力）
- [ ] find placeholder "Search..." clear（検索してクリア）
- [ ] find placeholder "Search..." --exact fill "keyword"
- [ ] find placeholder "text" --session <SESSION>
- [ ] find placeholder "text" -u <SERIAL>

### find enabled
- [ ] find enabled（有効な要素を検索）
- [ ] find enabled --first
- [ ] find enabled --last
- [ ] find enabled --nth 0
- [ ] find enabled --all
- [ ] find enabled --format text
- [ ] find enabled --format json
- [ ] find enabled tap（検索してタップ）
- [ ] find enabled --nth 2 tap
- [ ] find enabled --session <SESSION>
- [ ] find enabled -u <SERIAL>

### find disabled
- [ ] find disabled（無効な要素を検索）
- [ ] find disabled --first
- [ ] find disabled --last
- [ ] find disabled --nth 0
- [ ] find disabled --all
- [ ] find disabled --format text
- [ ] find disabled --format json
- [ ] find disabled --session <SESSION>
- [ ] find disabled -u <SERIAL>

### snapshot
- [ ] snapshot（基本的なスナップショット）
- [ ] snapshot --interactive（インタラクティブ要素のみ）
- [ ] snapshot --compact（空の構造要素を削除）
- [ ] snapshot --depth 2（ツリーの深さを制限）
- [ ] snapshot --depth 5
- [ ] snapshot --scope @eN（特定要素以下を対象）
- [ ] snapshot --scope "text"
- [ ] snapshot -o filename.txt（ファイルに出力）
- [ ] snapshot --format text
- [ ] snapshot --format json
- [ ] snapshot --no-scroll（スクロール無効）
- [ ] snapshot --max-scrolls 3
- [ ] snapshot --max-scrolls 10
- [ ] snapshot --scroll-delay 300
- [ ] snapshot --scroll-delay 1000
- [ ] snapshot --interactive --compact --depth 3
- [ ] snapshot --session <SESSION>
- [ ] snapshot -u <SERIAL>

## 待機関連コマンド

### wait
- [ ] wait visible @eN（要素が表示されるまで待機）
- [ ] wait visible "text"
- [ ] wait gone @eN（要素が消えるまで待機）
- [ ] wait gone "text"
- [ ] wait idle（UIアイドル待機）
- [ ] wait text "expected text"（テキストが画面に表示されるまで待機）
- [ ] wait visible @eN --timeout 10s
- [ ] wait visible @eN --timeout 60s
- [ ] wait visible @eN --timeout 30
- [ ] wait visible @eN --interval 500
- [ ] wait visible @eN --interval 1000
- [ ] wait visible @eN --timeout 30s --interval 500
- [ ] wait visible @eN --session <SESSION>
- [ ] wait visible @eN -u <SERIAL>

## 画面/記録関連コマンド

### screenshot
- [ ] screenshot（デフォルト: 現在ディレクトリにPNG保存）
- [ ] screenshot -o /path/to/file.png（ファイルパス指定）
- [ ] screenshot -o /path/to/directory/（ディレクトリ指定・自動生成）
- [ ] screenshot --format png
- [ ] screenshot --format jpeg
- [ ] screenshot -o /path/file.jpg --format jpeg
- [ ] screenshot --session <SESSION>
- [ ] screenshot -u <SERIAL>

### record
- [ ] record（Ctrl+Cまで録画）
- [ ] record -o /path/to/video.mp4（ファイルパス指定）
- [ ] record -o /path/to/directory/（ディレクトリ指定・自動生成）
- [ ] record --time-limit 10（10秒録画）
- [ ] record --time-limit 60
- [ ] record -o /path/file.mp4 --time-limit 30
- [ ] record --session <SESSION>
- [ ] record -u <SERIAL>

## ログ関連コマンド

### console (logcat)
- [ ] console（標準出力にストリーミング）
- [ ] console -o /path/to/logfile.txt（ファイルに出力）
- [ ] console --session <SESSION>
- [ ] console -u <SERIAL>

## アプリ管理関連コマンド

### app launch
- [ ] app launch com.example.app（アプリ起動）
- [ ] app launch com.example.app --format text
- [ ] app launch com.example.app --format json
- [ ] app launch com.example.app --session <SESSION>
- [ ] app launch com.example.app -u <SERIAL>

### app terminate
- [ ] app terminate com.example.app（アプリ終了）
- [ ] app terminate com.example.app --session <SESSION>
- [ ] app terminate com.example.app -u <SERIAL>

### app install
- [ ] app install /path/to/app.apk（Androidアプリインストール）
- [ ] app install /path/to/app.apk --session <SESSION>
- [ ] app install /path/to/app.apk -u <SERIAL>

### app uninstall
- [ ] app uninstall com.example.app（アプリアンインストール）
- [ ] app uninstall com.example.app --session <SESSION>
- [ ] app uninstall com.example.app -u <SERIAL>

### app list
- [ ] app list（アプリリスト表示）
- [ ] app list --format text
- [ ] app list --format json
- [ ] app list --session <SESSION>
- [ ] app list -u <SERIAL>

### app grant
- [ ] app grant camera --bundle com.example.app（権限付与）
- [ ] app grant location --bundle com.example.app
- [ ] app grant contacts --bundle com.example.app
- [ ] app grant camera --bundle com.example.app --session <SESSION>
- [ ] app grant camera --bundle com.example.app -u <SERIAL>

### app revoke
- [ ] app revoke camera --bundle com.example.app（権限削除）
- [ ] app revoke location --bundle com.example.app
- [ ] app revoke contacts --bundle com.example.app
- [ ] app revoke camera --bundle com.example.app --session <SESSION>
- [ ] app revoke camera --bundle com.example.app -u <SERIAL>

### app reset
- [ ] app reset camera --bundle com.example.app（権限リセット）
- [ ] app reset location --bundle com.example.app
- [ ] app reset contacts --bundle com.example.app
- [ ] app reset camera --bundle com.example.app --session <SESSION>
- [ ] app reset camera --bundle com.example.app -u <SERIAL>

## デバイス管理関連コマンド

### device list
- [ ] device list（デバイスリスト表示）
- [ ] device list --format text
- [ ] device list --format json
- [ ] device list -p android
- [ ] device list --session <SESSION>

### device boot (emulator)
- [ ] device boot "Pixel 7"（エミュレータ起動・名前指定）
- [ ] device boot "Pixel 7" -p android
- [ ] device boot "Pixel 7" --session <SESSION>

### device shutdown
- [ ] device shutdown -u <SERIAL>（エミュレータシャットダウン）
- [ ] device shutdown -u <SERIAL> --session <SESSION>

## セッション管理関連コマンド

### session list
- [ ] session list（セッションリスト表示）
- [ ] session list --session <SESSION>

### session show
- [ ] session show（セッション情報表示）
- [ ] session show --session <SESSION>

### session create
- [ ] session create --session my-session（セッション作成）

### session destroy
- [ ] session destroy --session my-session（セッション削除）

## グローバルオプション組み合わせパターン

### 一般的なコマンド
- [ ] --session <SESSION> 単独使用
- [ ] -u <SERIAL> 単独使用
- [ ] --session <SESSION> と -u <SERIAL> の組み合わせ
- [ ] AGENT_MOBILE_SESSION 環境変数でセッション指定

### device boot / device list 専用
- [ ] -p android 単独使用（device boot, device list）
- [ ] -p android と --session <SESSION> の組み合わせ

### ヘルプとバージョン
- [ ] --help フラグで各コマンドのヘルプ表示
- [ ] --version フラグでバージョン表示

## 出力形式オプション

- [ ] --format text（テキスト形式出力）
- [ ] --format json（JSON形式出力）
- [ ] JSON出力の構造検証
- [ ] テキスト出力の可読性確認

## Android 固有の要素タイプ

Android では iOS と異なる要素タイプが使用されます：

| Android | iOS (参考) |
|---------|-----------|
| Button | Button |
| EditText | TextField |
| TextView | StaticText |
| ImageView | Image |
| CheckBox | CheckBox |
| Switch | Switch |
| ProgressBar | ProgressIndicator |
| RecyclerView | CollectionView |
| ScrollView | ScrollView |

## サマリー

**検証対象パターン総数**: 約400項目
- UI操作コマンド: 90+パターン
- 要素情報取得: 110+パターン
- 待機処理: 20+パターン
- 画面/記録: 15+パターン
- ログ: 5+パターン
- アプリ管理: 50+パターン
- デバイス管理: 20+パターン
- セッション管理: 10+パターン
- グローバルオプション: 15+パターン
- 出力形式: 4パターン

## 注意事項

1. **ADB 接続**: Android エミュレータ/実機が `adb devices` で認識されていることを確認
2. **エミュレータ起動**: Android Studio または `emulator` コマンドでエミュレータを起動
3. **クリップボード**: Android では `device pbcopy/pbpaste` は未対応（iOS のみ）
4. **権限管理**: Android では `adb shell pm grant/revoke` を内部で使用
