# agent-mobile CLI 動作確認チェックリスト

このファイルは `agent-mobile` の idb 以外のトップレベルコマンドの動作確認用チェックリストです。

## 凡例
- [ ] 未確認
- [x] 確認済み
- [!] 問題あり（詳細をコメントに記載）

---

## 1. Gesture (ジェスチャー操作)

### iOS

- [ ] `agent-mobile gesture --tap` - 画面中央タップ（デフォルト）
- [ ] `agent-mobile gesture --tap 100,200` - 座標指定タップ
- [ ] `agent-mobile gesture --tap center` - 明示的に中央タップ
- [ ] `agent-mobile gesture --swipe up` - 上方向スワイプ
- [ ] `agent-mobile gesture --swipe down` - 下方向スワイプ
- [ ] `agent-mobile gesture --swipe 100,200,100,500` - 座標指定スワイプ
- [ ] `agent-mobile gesture --scroll up` - 上方向スクロール
- [ ] `agent-mobile gesture --scroll down` - 下方向スクロール
- [ ] `agent-mobile gesture --long-press 100,200` - ロングプレス
- [ ] `agent-mobile gesture --long-press 100,200 --duration 2.0` - 時間指定ロングプレス
- [ ] `agent-mobile gesture --swipe up --duration 0.5` - 時間指定スワイプ
- [ ] `agent-mobile gesture --tap -u <UDID>` - UDID指定でタップ

### Android

- [ ] `agent-mobile gesture --tap -p android` - タップ
- [ ] `agent-mobile gesture --tap 100,200 -p android` - 座標指定タップ
- [ ] `agent-mobile gesture --swipe up -p android` - スワイプ
- [ ] `agent-mobile gesture --swipe 100,200,100,500 -p android` - 座標指定スワイプ
- [ ] `agent-mobile gesture --scroll down -p android` - スクロール
- [ ] `agent-mobile gesture --long-press 100,200 -p android` - ロングプレス
- [ ] `agent-mobile gesture --long-press 100,200 --duration 2.0 -p android` - 時間指定ロングプレス

---

## 2. Keyboard (キーボード操作)

### iOS

- [ ] `agent-mobile keyboard --text "Hello World"` - テキスト入力
- [ ] `agent-mobile keyboard --text "日本語テスト"` - 日本語入力
- [ ] `agent-mobile keyboard --key enter` - Enterキー押下
- [ ] `agent-mobile keyboard --key delete` - Deleteキー押下
- [ ] `agent-mobile keyboard --key tab` - Tabキー押下
- [ ] `agent-mobile keyboard --key space` - Spaceキー押下
- [ ] `agent-mobile keyboard --key escape` - Escapeキー押下
- [ ] `agent-mobile keyboard --key up` - 上矢印キー押下
- [ ] `agent-mobile keyboard --key down` - 下矢印キー押下
- [ ] `agent-mobile keyboard --key left` - 左矢印キー押下
- [ ] `agent-mobile keyboard --key right` - 右矢印キー押下
- [ ] `agent-mobile keyboard --button home` - ホームボタン押下
- [ ] `agent-mobile keyboard --button lock` - ロックボタン押下
- [ ] `agent-mobile keyboard --button side` - サイドボタン押下
- [ ] `agent-mobile keyboard --clear` - テキストクリア

### Android

- [ ] `agent-mobile keyboard --text "Hello World" -p android` - テキスト入力
- [ ] `agent-mobile keyboard --key enter -p android` - Enterキー押下
- [ ] `agent-mobile keyboard --key delete -p android` - Deleteキー押下
- [ ] `agent-mobile keyboard --key tab -p android` - Tabキー押下
- [ ] `agent-mobile keyboard --key escape -p android` - Escapeキー押下
- [ ] `agent-mobile keyboard --button home -p android` - ホームボタン押下
- [ ] `agent-mobile keyboard --button back -p android` - 戻るボタン押下
- [ ] `agent-mobile keyboard --button menu -p android` - メニューボタン押下
- [ ] `agent-mobile keyboard --button power -p android` - 電源ボタン押下
- [ ] `agent-mobile keyboard --button volume-up -p android` - 音量アップ押下
- [ ] `agent-mobile keyboard --button volume-down -p android` - 音量ダウン押下
- [ ] `agent-mobile keyboard --button app_switch -p android` - アプリ切替ボタン押下
- [ ] `agent-mobile keyboard --clear -p android` - テキストクリア

---

## 3. Navigator (要素ナビゲーション)

### iOS

- [ ] `agent-mobile navigator --list` - 全要素リスト表示
- [ ] `agent-mobile navigator --list -o json` - JSON形式で全要素リスト
- [ ] `agent-mobile navigator --find "Button"` - テキストで要素検索
- [ ] `agent-mobile navigator --find-type "Button"` - タイプで要素検索
- [ ] `agent-mobile navigator --find-id "button_id"` - IDで要素検索
- [ ] `agent-mobile navigator --find "Submit" --tap` - 要素を見つけてタップ
- [ ] `agent-mobile navigator --find "TextField" --enter-text "test"` - 要素を見つけてテキスト入力

### Android

- [ ] `agent-mobile navigator --list -p android` - 全要素リスト表示
- [ ] `agent-mobile navigator --list -p android -o json` - JSON形式で全要素リスト
- [ ] `agent-mobile navigator --find "Button" -p android` - テキストで要素検索
- [ ] `agent-mobile navigator --find-type "Button" -p android` - タイプで要素検索
- [ ] `agent-mobile navigator --find-id "button_id" -p android` - IDで要素検索
- [ ] `agent-mobile navigator --find "Submit" --tap -p android` - 要素を見つけてタップ
- [ ] `agent-mobile navigator --find "EditText" --enter-text "test" -p android` - 要素を見つけてテキスト入力

---

## 4. Screen (画面分析)

### iOS

- [ ] `agent-mobile screen --dump` - アクセシビリティツリー全体をダンプ
- [ ] `agent-mobile screen --summary` - 画面サマリー表示
- [ ] `agent-mobile screen --summary -o json` - JSON形式でサマリー
- [ ] `agent-mobile screen --hints` - インタラクティブ要素のヒント表示
- [ ] `agent-mobile screen --hints -o json` - JSON形式でヒント

### Android

- [ ] `agent-mobile screen --dump -p android` - UIヒエラルキーダンプ
- [ ] `agent-mobile screen --summary -p android` - 画面サマリー表示
- [ ] `agent-mobile screen --summary -p android -o json` - JSON形式でサマリー
- [ ] `agent-mobile screen --hints -p android` - インタラクティブ要素のヒント表示

---

## 5. App (アプリ管理)

### iOS

- [ ] `agent-mobile app --list` - インストール済みアプリ一覧
- [ ] `agent-mobile app --list -o json` - JSON形式でアプリ一覧
- [ ] `agent-mobile app --launch com.apple.mobilesafari` - アプリ起動
- [ ] `agent-mobile app --terminate com.apple.mobilesafari` - アプリ終了
- [ ] `agent-mobile app --install /path/to/app.app` - アプリインストール
- [ ] `agent-mobile app --uninstall com.example.app` - アプリアンインストール

### Android

- [ ] `agent-mobile app --list -p android` - インストール済みアプリ一覧
- [ ] `agent-mobile app --list -p android -o json` - JSON形式でアプリ一覧
- [ ] `agent-mobile app --launch com.android.settings -p android` - アプリ起動
- [ ] `agent-mobile app --terminate com.android.settings -p android` - アプリ終了
- [ ] `agent-mobile app --install /path/to/app.apk -p android` - アプリインストール
- [ ] `agent-mobile app --uninstall com.example.app -p android` - アプリアンインストール

---

## 6. Device (デバイス管理)

### iOS

- [ ] `agent-mobile device --list` - デバイス/シミュレータ一覧
- [ ] `agent-mobile device --list -o json` - JSON形式でデバイス一覧
- [ ] `agent-mobile device --boot "iPhone 15"` - 名前指定でシミュレータ起動
- [ ] `agent-mobile device --boot <UDID>` - UDID指定でシミュレータ起動
- [ ] `agent-mobile device --shutdown -u <UDID>` - シミュレータ停止

### Android

- [ ] `agent-mobile device --list -p android` - デバイス/エミュレータ一覧
- [ ] `agent-mobile device --list -p android -o json` - JSON形式でデバイス一覧
- [ ] `agent-mobile device --boot "Pixel_8_API_34" -p android` - エミュレータ起動
- [ ] `agent-mobile device --boot "Pixel_8_API_34" --headless -p android` - ヘッドレスで起動
- [ ] `agent-mobile device --shutdown -u <serial> -p android` - エミュレータ停止

---

## 7. Clipboard (クリップボード)

### iOS

- [ ] `agent-mobile clipboard --copy "Test text"` - クリップボードにコピー
- [ ] `agent-mobile clipboard --paste` - クリップボードからペースト
- [ ] `agent-mobile clipboard --copy "Test" -u <UDID>` - UDID指定でコピー

### Android

- [ ] `agent-mobile clipboard --copy "Test text" -p android` - エラー確認（非対応）
- [ ] `agent-mobile clipboard --paste -p android` - エラー確認（非対応）

---

## 8. Privacy (権限管理)

### iOS

- [ ] `agent-mobile privacy --grant camera -b com.example.app` - カメラ権限付与
- [ ] `agent-mobile privacy --grant location -b com.example.app` - 位置情報権限付与
- [ ] `agent-mobile privacy --grant microphone -b com.example.app` - マイク権限付与
- [ ] `agent-mobile privacy --grant photos -b com.example.app` - 写真権限付与
- [ ] `agent-mobile privacy --grant contacts -b com.example.app` - 連絡先権限付与
- [ ] `agent-mobile privacy --revoke camera -b com.example.app` - カメラ権限取消
- [ ] `agent-mobile privacy --reset camera -b com.example.app` - カメラ権限リセット

### Android

- [ ] `agent-mobile privacy --grant camera -b com.example.app -p android` - カメラ権限付与
- [ ] `agent-mobile privacy --grant location -b com.example.app -p android` - 位置情報権限付与
- [ ] `agent-mobile privacy --grant microphone -b com.example.app -p android` - マイク権限付与
- [ ] `agent-mobile privacy --grant storage -b com.example.app -p android` - ストレージ権限付与
- [ ] `agent-mobile privacy --grant contacts -b com.example.app -p android` - 連絡先権限付与
- [ ] `agent-mobile privacy --revoke camera -b com.example.app -p android` - カメラ権限取消

---

## 9. Accessibility (アクセシビリティ監査)

### iOS

- [ ] `agent-mobile accessibility --audit` - アクセシビリティ監査実行
- [ ] `agent-mobile accessibility --audit -o json` - JSON形式で監査結果
- [ ] `agent-mobile accessibility --audit -u <UDID>` - UDID指定で監査

### Android

- [ ] `agent-mobile accessibility --audit -p android` - アクセシビリティ監査実行
- [ ] `agent-mobile accessibility --audit -p android -o json` - JSON形式で監査結果

---

## 10. 共通オプション確認

- [ ] プラットフォーム自動検出が正しく動作する
- [ ] `-p ios` オプションが正しく動作する
- [ ] `-p android` オプションが正しく動作する
- [ ] `-u/--udid` オプションが正しくデバイス指定できる
- [ ] `-o/--output json` オプションが正しくJSON出力する
- [ ] `-o/--output human` オプションが正しく人間可読出力する

---

## 11. エラーケース確認

- [ ] 無効なプラットフォーム指定時のエラーメッセージ
- [ ] 存在しないUDID指定時のエラーメッセージ
- [ ] 必須オプション未指定時のエラーメッセージ
- [ ] 無効な座標形式指定時のエラーメッセージ
- [ ] デバイス未接続時のエラーメッセージ

---

## 確認環境

### iOS
- macOS バージョン:
- Xcode バージョン:
- シミュレータ:
- idb_companion バージョン:

### Android
- Android Studio バージョン:
- エミュレータ:
- adb バージョン:

---

## 確認日・確認者

- 確認日:
- 確認者:
- 備考:

---

## 問題/課題メモ

<!-- 発見した問題や課題をここに記載 -->

