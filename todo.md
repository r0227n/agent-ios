# agent-mobile 実装済み機能リスト（オプション含む全パターン）

## UI操作関連コマンド

### tap
- [ ] tap @eN（参照でタップ）
- [ ] tap "text"（テキストでタップ）
- [ ] tap x,y（座標でタップ）
- [ ] tap home/back/enter（キーでタップ）
- [ ] tap @eN --duration 0.5
- [ ] tap "text" --duration 0.5
- [ ] tap x,y --duration 0.5
- [ ] tap @eN -p ios
- [ ] tap @eN -p android
- [ ] tap @eN --session <SESSION>
- [ ] tap @eN -u <UDID>

### check
- [ ] check @eN（参照でチェック）
- [ ] check "text"（テキストでチェック）
- [ ] check @eN -p ios
- [ ] check @eN -p android
- [ ] check @eN --session <SESSION>
- [ ] check @eN -u <UDID>

### uncheck
- [ ] uncheck @eN（参照でアンチェック）
- [ ] uncheck "text"（テキストでアンチェック）
- [ ] uncheck @eN -p ios
- [ ] uncheck @eN -p android
- [ ] uncheck @eN --session <SESSION>
- [ ] uncheck @eN -u <UDID>

### select
- [ ] select @eN "value"（参照でピッカーから選択）
- [ ] select "text" "value"（テキストでピッカーから選択）
- [ ] select @eN "value" --max-swipes 5
- [ ] select @eN "value" --max-swipes 20
- [ ] select @eN "value" -p ios
- [ ] select @eN "value" -p android
- [ ] select @eN "value" --session <SESSION>
- [ ] select @eN "value" -u <UDID>

### long-press
- [ ] long-press @eN（参照で長押し）
- [ ] long-press "text"（テキストで長押し）
- [ ] long-press x,y（座標で長押し）
- [ ] long-press center（中央で長押し）
- [ ] long-press @eN --duration 1
- [ ] long-press @eN --duration 2
- [ ] long-press @eN -p ios
- [ ] long-press @eN -p android
- [ ] long-press @eN --session <SESSION>
- [ ] long-press @eN -u <UDID>

### fill
- [ ] fill @eN "text"（参照でテキストを入力）
- [ ] fill "placeholder" "text"（プレースホルダでテキストを入力）
- [ ] fill @eN "text" -p ios
- [ ] fill @eN "text" -p android
- [ ] fill @eN "text" --session <SESSION>
- [ ] fill @eN "text" -u <UDID>

### type
- [ ] type "text"（フォーカス済みフィールドにテキストを追記）
- [ ] type "text" -p ios
- [ ] type "text" -p android
- [ ] type "text" --session <SESSION>
- [ ] type "text" -u <UDID>

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
- [ ] swipe up -p ios
- [ ] swipe up -p android
- [ ] swipe up --session <SESSION>
- [ ] swipe up -u <UDID>

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
- [ ] scroll up -p ios
- [ ] scroll up -p android
- [ ] scroll up --session <SESSION>
- [ ] scroll up -u <UDID>

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
- [ ] get text @eN -p ios
- [ ] get text @eN -p android
- [ ] get text @eN --session <SESSION>
- [ ] get text @eN -u <UDID>

### is
- [ ] is visible @eN（可視性を確認）
- [ ] is exists @eN（存在確認）
- [ ] is enabled @eN（有効確認）
- [ ] is disabled @eN（無効確認）
- [ ] is interactive @eN（インタラクティブ確認）
- [ ] is checked @eN（チェック状態確認）
- [ ] is visible "text"
- [ ] is visible @eN -p ios
- [ ] is visible @eN -p android
- [ ] is visible @eN --session <SESSION>
- [ ] is visible @eN -u <UDID>

### find type
- [ ] find type Button（ボタン型を検索）
- [ ] find type TextField（テキストフィールド型を検索）
- [ ] find type Switch
- [ ] find type Text
- [ ] find type Image
- [ ] find type --first（最初の要素のみ）
- [ ] find type Button --last（最後の要素のみ）
- [ ] find type Button --nth 0（N番目の要素）
- [ ] find type Button --all（すべて取得）
- [ ] find type Button --format text
- [ ] find type Button --format json
- [ ] find type Button -p ios
- [ ] find type Button -p android
- [ ] find type Button --session <SESSION>
- [ ] find type Button -u <UDID>

### find text
- [ ] find text "search term"（テキストで検索）
- [ ] find text "text" --first
- [ ] find text "text" --last
- [ ] find text "text" --nth 1
- [ ] find text "text" --all
- [ ] find text "text" --format text
- [ ] find text "text" --format json
- [ ] find text "text" -p ios
- [ ] find text "text" -p android
- [ ] find text "text" --session <SESSION>
- [ ] find text "text" -u <UDID>

### find label
- [ ] find label "label text"（ラベルで検索）
- [ ] find label "label" --first
- [ ] find label "label" --last
- [ ] find label "label" --nth 0
- [ ] find label "label" --all
- [ ] find label "label" --format text
- [ ] find label "label" --format json
- [ ] find label "label" -p ios
- [ ] find label "label" -p android
- [ ] find label "label" --session <SESSION>
- [ ] find label "label" -u <UDID>

### find placeholder
- [ ] find placeholder "placeholder text"（プレースホルダで検索）
- [ ] find placeholder "text" --first
- [ ] find placeholder "text" --last
- [ ] find placeholder "text" --nth 0
- [ ] find placeholder "text" --all
- [ ] find placeholder "text" --format text
- [ ] find placeholder "text" --format json
- [ ] find placeholder "text" -p ios
- [ ] find placeholder "text" -p android
- [ ] find placeholder "text" --session <SESSION>
- [ ] find placeholder "text" -u <UDID>

### find enabled
- [ ] find enabled（有効な要素を検索）
- [ ] find enabled --first
- [ ] find enabled --last
- [ ] find enabled --nth 0
- [ ] find enabled --all
- [ ] find enabled --format text
- [ ] find enabled --format json
- [ ] find enabled -p ios
- [ ] find enabled -p android
- [ ] find enabled --session <SESSION>
- [ ] find enabled -u <UDID>

### find disabled
- [ ] find disabled（無効な要素を検索）
- [ ] find disabled --first
- [ ] find disabled --last
- [ ] find disabled --nth 0
- [ ] find disabled --all
- [ ] find disabled --format text
- [ ] find disabled --format json
- [ ] find disabled -p ios
- [ ] find disabled -p android
- [ ] find disabled --session <SESSION>
- [ ] find disabled -u <UDID>

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
- [ ] snapshot -p ios
- [ ] snapshot -p android
- [ ] snapshot --session <SESSION>
- [ ] snapshot -u <UDID>

## 待機関連コマンド

### wait
- [ ] wait visible @eN（要素が表示されるまで待機）
- [ ] wait visible "text"
- [ ] wait gone @eN（要素が消えるまで待機）
- [ ] wait gone "text"
- [ ] wait idle（UIアイドル待機）
- [ ] wait text @eN "expected text"（テキストが一致するまで待機）
- [ ] wait visible @eN --timeout 10s
- [ ] wait visible @eN --timeout 60s
- [ ] wait visible @eN --timeout 30
- [ ] wait visible @eN --interval 500
- [ ] wait visible @eN --interval 1000
- [ ] wait visible @eN --timeout 30s --interval 500
- [ ] wait visible @eN -p ios
- [ ] wait visible @eN -p android
- [ ] wait visible @eN --session <SESSION>
- [ ] wait visible @eN -u <UDID>

## 画面/記録関連コマンド

### screenshot
- [ ] screenshot（デフォルト: 現在ディレクトリにPNG保存）
- [ ] screenshot -o /path/to/file.png（ファイルパス指定）
- [ ] screenshot -o /path/to/directory/（ディレクトリ指定・自動生成）
- [ ] screenshot --format png
- [ ] screenshot --format jpeg
- [ ] screenshot -o /path/file.jpg --format jpeg
- [ ] screenshot -p ios
- [ ] screenshot -p android
- [ ] screenshot --session <SESSION>
- [ ] screenshot -u <UDID>

### record
- [ ] record（Ctrl+Cまで録画）
- [ ] record -o /path/to/video.mp4（ファイルパス指定）
- [ ] record -o /path/to/directory/（ディレクトリ指定・自動生成）
- [ ] record --time-limit 10（10秒録画）
- [ ] record --time-limit 60
- [ ] record -o /path/file.mp4 --time-limit 30
- [ ] record -p ios
- [ ] record -p android
- [ ] record --session <SESSION>
- [ ] record -u <UDID>

## ログ関連コマンド

### console
- [ ] console（標準出力にストリーミング）
- [ ] console -o /path/to/logfile.txt（ファイルに出力）
- [ ] console -p ios
- [ ] console -p android
- [ ] console --session <SESSION>
- [ ] console -u <UDID>

## アプリ管理関連コマンド

### app launch
- [ ] app launch com.example.app（アプリ起動）
- [ ] app launch com.example.app --format text
- [ ] app launch com.example.app --format json
- [ ] app launch com.example.app -p ios
- [ ] app launch com.example.app -p android
- [ ] app launch com.example.app --session <SESSION>
- [ ] app launch com.example.app -u <UDID>

### app terminate
- [ ] app terminate com.example.app（アプリ終了）
- [ ] app terminate com.example.app -p ios
- [ ] app terminate com.example.app -p android
- [ ] app terminate com.example.app --session <SESSION>
- [ ] app terminate com.example.app -u <UDID>

### app install
- [ ] app install /path/to/app.app（iOSアプリインストール）
- [ ] app install /path/to/app.ipa
- [ ] app install /path/to/app.apk（Androidアプリインストール）
- [ ] app install /path/to/app.apk -p android
- [ ] app install /path/to/app.app --session <SESSION>
- [ ] app install /path/to/app.app -u <UDID>

### app uninstall
- [ ] app uninstall com.example.app（アプリアンインストール）
- [ ] app uninstall com.example.app -p ios
- [ ] app uninstall com.example.app -p android
- [ ] app uninstall com.example.app --session <SESSION>
- [ ] app uninstall com.example.app -u <UDID>

### app list
- [ ] app list（アプリリスト表示）
- [ ] app list --format text
- [ ] app list --format json
- [ ] app list -p ios
- [ ] app list -p android
- [ ] app list --session <SESSION>
- [ ] app list -u <UDID>

### app grant
- [ ] app grant camera --bundle com.example.app（権限付与）
- [ ] app grant location --bundle com.example.app
- [ ] app grant contacts --bundle com.example.app
- [ ] app grant camera --bundle com.example.app -p ios
- [ ] app grant camera --bundle com.example.app -p android
- [ ] app grant camera --bundle com.example.app --session <SESSION>
- [ ] app grant camera --bundle com.example.app -u <UDID>

### app revoke
- [ ] app revoke camera --bundle com.example.app（権限削除）
- [ ] app revoke location --bundle com.example.app
- [ ] app revoke contacts --bundle com.example.app
- [ ] app revoke camera --bundle com.example.app -p ios
- [ ] app revoke camera --bundle com.example.app -p android
- [ ] app revoke camera --bundle com.example.app --session <SESSION>
- [ ] app revoke camera --bundle com.example.app -u <UDID>

### app reset
- [ ] app reset camera --bundle com.example.app（権限リセット）
- [ ] app reset location --bundle com.example.app
- [ ] app reset contacts --bundle com.example.app
- [ ] app reset camera --bundle com.example.app -p ios
- [ ] app reset camera --bundle com.example.app -p android
- [ ] app reset camera --bundle com.example.app --session <SESSION>
- [ ] app reset camera --bundle com.example.app -u <UDID>

## デバイス管理関連コマンド

### device list
- [ ] device list（デバイスリスト表示）
- [ ] device list --format text
- [ ] device list --format json
- [ ] device list -p ios
- [ ] device list -p android
- [ ] device list --session <SESSION>

### device boot
- [ ] device boot iPhone15（シミュレータ起動・名前指定）
- [ ] device boot 12AB34CD-EF56-GHIJ-KLMN-OPQRSTUVWXYZ（UDID指定）
- [ ] device boot "iPhone 15" -p ios
- [ ] device boot --session <SESSION>

### device shutdown
- [ ] device shutdown（シミュレータシャットダウン）
- [ ] device shutdown -p ios
- [ ] device shutdown --session <SESSION>
- [ ] device shutdown -u <UDID>

### device pbcopy
- [ ] device pbcopy "text"（クリップボードにコピー）
- [ ] device pbcopy "text" -p ios
- [ ] device pbcopy "text" -p android
- [ ] device pbcopy "text" --session <SESSION>
- [ ] device pbcopy "text" -u <UDID>

### device pbpaste
- [ ] device pbpaste（クリップボードから取得）
- [ ] device pbpaste -p ios
- [ ] device pbpaste -p android
- [ ] device pbpaste --session <SESSION>
- [ ] device pbpaste -u <UDID>

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

- [ ] --session <SESSION> 単独使用
- [ ] -p ios 単独使用
- [ ] -p android 単独使用
- [ ] -u <UDID> 単独使用
- [ ] --session <SESSION> と -p ios の組み合わせ
- [ ] --session <SESSION> と -p android の組み合わせ
- [ ] --session <SESSION> と -u <UDID> の組み合わせ
- [ ] -p ios と -u <UDID> の組み合わせ
- [ ] -p android と -u <UDID> の組み合わせ
- [ ] --session <SESSION> と -p ios と -u <UDID> の組み合わせ
- [ ] --session <SESSION> と -p android と -u <UDID> の組み合わせ
- [ ] AGENT_MOBILE_SESSION 環境変数でセッション指定
- [ ] --help フラグで各コマンドのヘルプ表示
- [ ] --version フラグでバージョン表示

## 出力形式オプション

- [ ] --format text（テキスト形式出力）
- [ ] --format json（JSON形式出力）
- [ ] JSON出力の構造検証
- [ ] テキスト出力の可読性確認

## サマリー

**検証対象パターン総数**: 450+以上のテストケース
- UI操作コマンド: 70+パターン
- 要素情報取得: 80+パターン
- 待機処理: 20+パターン
- 画面/記録: 15+パターン
- ログ: 10+パターン
- アプリ管理: 50+パターン
- デバイス管理: 35+パターン
- セッション管理: 10+パターン
- グローバルオプション: 15+パターン
- 出力形式: 4パターン

※ `agent-mobile idb` は削除予定のため検証対象外
