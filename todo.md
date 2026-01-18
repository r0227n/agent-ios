# agent-mobile idb コマンド実装状況

Python版idbから移植が完了していないコマンドのリストです。

## 実装済みコマンド ✅

### 基本コマンド
- [x] list-targets - 接続済み/利用可能なターゲット一覧
- [x] launch - アプリケーションを起動
- [x] kill - idb daemonを停止
- [x] screenshot - ターゲットデバイスのスクリーンショットを取得
- [x] focus - シミュレータウィンドウを前面に表示
- [x] log - ターゲットまたはcompanionからログを取得

### アプリ管理 (App Management)
- [x] app install - アプリケーション(.app/.ipa)をインストール
- [x] app uninstall - アプリケーションをアンインストール
- [x] terminate - 実行中のアプリケーションを終了
- [x] list-apps - インストール済みアプリ一覧

### ファイル操作 (File Operations)
- [x] file ls - アプリケーションコンテナ内のパス一覧
- [x] file mkdir - アプリケーションコンテナ内にディレクトリ作成
- [x] file mv - アプリケーションコンテナ内でパスを移動
- [x] file rm - コンテナ内のアイテムを削除
- [x] file push - ローカルマシンからターゲットへファイルをコピー
- [x] file pull - アプリケーションコンテナからローカルマシンへファイルをコピー
- [x] file tail - リモートファイルをstdoutにtail出力
- [x] file read - リモートファイルの内容を読み取りstdoutに出力
- [x] file write - stdinを読み取りリモートファイルに書き込み

### クラッシュログ (Crash Logs)
- [x] crash list - 利用可能なクラッシュ一覧
- [x] crash show - クラッシュログを取得
- [x] crash delete - クラッシュログを削除

### デバイス設定 (Settings)
- [x] settings set - 設定を変更
- [x] settings get - 設定値を取得
- [x] settings list-locale - 利用可能なロケール識別子一覧

### 位置情報 (Location)
- [x] location set-location - シミュレータの位置を設定（緯度/経度）

### 権限管理 (Permissions)
- [x] approve - アプリの権限を承認
  - ⚠️ Known Issue: photos/camera/contacts fail due to idb_companion SQLite schema mismatch
  - ✅ location/notification/url work correctly
- [x] revoke - アプリの権限を取り消し
  - ✅ All permission types work correctly

### 通知・URL (Notifications & URLs)
- [x] notification send-notification - 通知を送信
- [x] url open - URLを開く

### XCTest関連
- [x] xctest-list - インストール済みテストバンドル一覧
- [x] xctest-list-bundle - インストール済みテストバンドル内のテスト一覧
- [x] xctest-run - テスト実行（app/ui/logic）
- [x] xctest install - xctestをインストール

### HID (Human Interface Device) - 画面操作
- [x] hid tap - 画面の座標をタップ
- [x] hid button - ボタンを1回押す
- [x] hid key - キーコードを短押し
- [x] hid key-sequence - キーコードのシーケンスを短押し
- [x] hid text - テキストを入力
- [x] hid swipe - ある点から別の点へスワイプ

### デバッグ関連 (Debugging)
- [x] debugserver start - アプリのデバッグサーバーを起動
- [x] debugserver stop - デバッグサーバーを停止
- [x] debugserver status - デバッグサーバーの状態を取得
- [x] dap - VSCode DAPプロトコルを使用したデバッグサーバーを生成

### メディア・写真・ビデオ (Media)
- [x] media add-media - 写真/ビデオをターゲットに追加
- [x] photos clear - すべての写真をクリア
- [x] video record-video - ターゲット画面をmp4ビデオファイルに録画
- [x] video stream - ターゲットからraw H264をストリーム配信

### ターゲット/シミュレータ管理 (Target Management)
- [x] target connect - companionに接続
- [x] target disconnect - companionを切断
- [x] target describe - ターゲットの詳細情報を表示
- [x] target create - iOSシミュレータを作成
- [x] target boot - シミュレータを起動
- [x] target shutdown - シミュレータをシャットダウン
- [x] target erase - シミュレータを消去
- [x] target clone - シミュレータをクローン
- [x] target delete - シミュレータを削除（--allフラグでdelete-all機能も含む）

### アクセシビリティ (Accessibility)
- [x] accessibility describe-all - 画面全体のアクセシビリティ情報を説明
- [x] accessibility describe-point - 画面上のポイントのアクセシビリティ情報を説明

### データ管理 (Data Management)
- [x] contacts update - データベースから連絡先を更新
- [x] contacts clear - すべての連絡先をクリア
- [x] keychain clear-keychain - ターゲットのキーチェーンをクリア

### システム (System)
- [x] memory simulate-memory-warning - メモリ警告をシミュレート

---

## 未実装コマンド

### プロファイリング (Profiling)
- [ ] instruments - デバイス上でinstruments/プロファイリングを実行
- [ ] xctrace record - トレースを記録

### バイナリ・デバッグシンボル (Binary & Debug Symbols)
- [ ] dsym install - dSYM（デバッグシンボル）をインストール
- [ ] dylib install - dylibをインストール
- [ ] framework install - Frameworkバンドルをインストール

### インタラクティブシェル (Interactive Shell)
- [ ] shell - 複数のIDBコマンドをチェーン実行できるインタラクティブシェル

---

## 統計

- **実装済み**: 62コマンド
- **未実装**: 6コマンド
- **進捗率**: 約91%

## 参考

- Python実装: `idb/idb/cli/commands/`
- Rust実装: `src/cli/idb/`
- gRPC定義: `proto/idb.proto`
