# agent-mobile idb コマンド実装状況

Python版idbから移植が完了していないコマンドのリストです。

## 実装済みコマンド ✅

- [x] list-targets - 接続済み/利用可能なターゲット一覧
- [x] launch - アプリケーションを起動
- [x] kill - idb daemonを停止
- [x] screenshot - ターゲットデバイスのスクリーンショットを取得
- [x] focus - シミュレータウィンドウを前面に表示
- [x] log - ターゲットまたはcompanionからログを取得
- [x] app install - アプリケーション(.app/.ipa)をインストール
- [x] app uninstall - アプリケーションをアンインストール

---

## 未実装コマンド

### 1. アプリ管理 (App Management)

- [ ] app terminate - 実行中のアプリケーションを終了
- [ ] app list-apps - ターゲットにインストールされているアプリ一覧

### 2. ファイル操作 (File Operations)

- [ ] file list / ls - アプリケーションコンテナ内のパス一覧
- [ ] file mkdir - アプリケーションコンテナ内にディレクトリ作成
- [ ] file move / mv - アプリケーションコンテナ内でパスを移動
- [ ] file remove / rm - コンテナ内のアイテムを削除
- [ ] file push - ローカルマシンからターゲットへファイルをコピー
- [ ] file pull - アプリケーションコンテナからローカルマシンへファイルをコピー
- [ ] file read / show - リモートファイルの内容を読み取りstdoutに出力
- [ ] file write - stdinを読み取りリモートファイルに書き込み
- [ ] file tail - リモートファイルをstdoutにtail出力

### 3. HID (Human Interface Device) - 画面操作

- [ ] hid tap - 画面の座標をタップ
- [ ] hid button - ボタンを1回押す
- [ ] hid key - キーコードを短押し
- [ ] hid key-sequence - キーコードのシーケンスを短押し
- [ ] hid text - テキストを入力
- [ ] hid swipe - ある点から別の点へスワイプ

### 4. デバッグ・テスト (Testing & Debugging)

#### XCTest関連
- [ ] xctest install - xctestをインストール
- [ ] xctest list - インストール済みテストバンドル一覧
- [ ] xctest list-bundle - インストール済みテストバンドル内のテスト一覧
- [ ] xctest run app - アプリテストを実行
- [ ] xctest run ui - UIテストを実行
- [ ] xctest run logic - ロジックテストを実行

#### デバッグサーバー関連
- [ ] debugserver start - アプリのデバッグサーバーを起動
- [ ] debugserver stop - デバッグサーバーを停止
- [ ] debugserver status - デバッグサーバーの状態を取得
- [ ] dap - VSCode DAPプロトコルを使用した新しいデバッグサーバーを生成

### 5. クラッシュログ (Crash Logs)

- [ ] crash list - 利用可能なクラッシュ一覧
- [ ] crash show - クラッシュログを取得
- [ ] crash delete - クラッシュログを削除

### 6. メディア・写真・ビデオ (Media)

- [ ] media add-media - 写真/ビデオをターゲットに追加
- [ ] photos clear - すべての写真をクリア
- [ ] video record-video - ターゲット画面をmp4ビデオファイルに録画
- [ ] video stream - ターゲットからraw H264をストリーム配信

### 7. デバイス設定 (Settings)

- [ ] settings set - 設定を変更
- [ ] settings get - 設定値を取得
- [ ] settings list locale - 利用可能なロケール識別子一覧

### 8. 位置情報 (Location)

- [ ] location set-location - シミュレータの位置を設定（緯度/経度）

### 9. 権限管理 (Permissions)

- [ ] approve - アプリの権限を承認
- [ ] revoke - アプリの権限を取り消し

### 10. 通知・URL (Notifications & URLs)

- [ ] notification send-notification - URLを開く/通知を送信
- [ ] url open - URLを開く

### 11. シミュレータ管理 (Simulator Management)

#### 接続管理
- [ ] target connect - companionに接続
- [ ] target disconnect - companionを切断
- [ ] target describe - ターゲットの詳細情報を表示

#### ライフサイクル管理
- [ ] target create - iOSシミュレータを作成
- [ ] target boot - シミュレータを起動
- [ ] target shutdown - シミュレータをシャットダウン
- [ ] target erase - シミュレータを消去
- [ ] target clone - シミュレータをクローン
- [ ] target delete - シミュレータを削除
- [ ] target delete-all - すべてのシミュレータを削除

### 12. プロファイリング (Profiling)

- [ ] instruments - デバイス上でinstruments/プロファイリングを実行
- [ ] xctrace record - トレースを記録

### 13. システム・その他 (System & Miscellaneous)

#### アクセシビリティ
- [ ] accessibility describe-all - 画面全体のアクセシビリティ情報を説明
- [ ] accessibility describe-point - 画面上のポイントのアクセシビリティ情報を説明

#### データ管理
- [ ] contacts update - データベースから連絡先を更新
- [ ] contacts clear - すべての連絡先をクリア
- [ ] keychain clear-keychain - ターゲットのキーチェーンをクリア

#### システム
- [ ] memory simulate-memory-warning - メモリ警告をシミュレート

#### バイナリ・デバッグシンボル
- [ ] dsym install - dSYM（デバッグシンボル）をインストール
- [ ] dylib install - dylibをインストール
- [ ] framework install - Frameworkバンドルをインストール

#### インタラクティブ
- [ ] shell - 複数のIDBコマンドをチェーン実行できるインタラクティブシェル

---

## 統計

- **実装済み**: 6コマンド
- **未実装**: 90+コマンド
- **カテゴリー**: 14カテゴリー

## 参考

- Python実装: `idb/idb/cli/commands/`
- Rust実装: `src/cli/idb/`
- gRPC定義: `proto/idb.proto`
