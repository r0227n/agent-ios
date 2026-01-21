# Element Type Reference

agent-mobile で使用する UI 要素タイプの対応表。

## 概要

agent-mobile は iOS と Android の両プラットフォームに対応しており、
各プラットフォームの API から取得される要素タイプを統一された表示名に正規化します。

### 変換ルール

| プラットフォーム | API 型名形式 | 変換ルール |
|----------------|-------------|-----------|
| iOS | `AX{TypeName}` | プレフィックス `AX` を除去 |
| Android | `{package}.{ClassName}` | パッケージ名を除去し、クラス名のみ抽出 |

## 要素タイプ対応表

| iOS API 型名 | Android API 型名 | agent-mobile 表示名 | 説明 |
|-------------|-----------------|-------------------|------|
| AXApplication | - | Application | アプリケーション |
| AXWindow | - | Window | ウィンドウ |
| AXButton | android.widget.Button | Button | ボタン |
| AXStaticText | android.widget.TextView | StaticText / TextView | 静的テキスト |
| AXTextField | android.widget.EditText | TextField / EditText | テキスト入力フィールド |
| AXSecureTextField | - | SecureTextField | パスワード入力フィールド |
| AXTextView | - | TextView | 複数行テキストビュー |
| AXImage | android.widget.ImageView | Image / ImageView | 画像 |
| - | android.widget.ImageButton | ImageButton | 画像ボタン |
| AXScrollView | android.widget.ScrollView | ScrollView | スクロールビュー |
| - | android.widget.HorizontalScrollView | HorizontalScrollView | 横スクロールビュー |
| AXTable | android.widget.ListView | Table / ListView | テーブル / リストビュー |
| AXCell | - | Cell | テーブルセル |
| AXNavigationBar | - | NavigationBar | ナビゲーションバー |
| AXTabBar | - | TabBar | タブバー |
| AXToolbar | androidx.appcompat.widget.Toolbar | Toolbar | ツールバー |
| AXSearchField | - | SearchField | 検索フィールド |
| AXSlider | android.widget.SeekBar | Slider / SeekBar | スライダー |
| AXSwitch | android.widget.Switch | Switch | スイッチ |
| - | android.widget.ToggleButton | ToggleButton | トグルボタン |
| - | android.widget.CheckBox | CheckBox | チェックボックス |
| - | android.widget.RadioButton | RadioButton | ラジオボタン |
| AXProgressIndicator | android.widget.ProgressBar | ProgressIndicator / ProgressBar | プログレスインジケーター |
| AXActivityIndicator | - | ActivityIndicator | アクティビティインジケーター |
| AXSegmentedControl | - | SegmentedControl | セグメントコントロール |
| AXPicker | android.widget.Spinner | Picker / Spinner | ピッカー / スピナー |
| AXDatePicker | - | DatePicker | 日付ピッカー |
| AXPageIndicator | - | PageIndicator | ページインジケーター |
| AXLink | - | Link | リンク |
| AXHeading | - | Heading | 見出し |
| AXGroup | android.view.ViewGroup | Group / ViewGroup | グループ |
| AXList | - | List | リスト |
| - | android.widget.GridView | GridView | グリッドビュー |
| AXCollectionView | androidx.recyclerview.widget.RecyclerView | CollectionView / RecyclerView | コレクションビュー |
| AXWebView | android.webkit.WebView | WebView | ウェブビュー |
| AXMap | - | Map | 地図 |
| AXAlert | - | Alert | アラート |
| AXSheet | - | Sheet | シート |
| AXPopover | - | Popover | ポップオーバー |
| AXMenu | - | Menu | メニュー |
| AXMenuItem | - | MenuItem | メニュー項目 |
| - | android.view.View | View | ビュー |
| - | android.widget.LinearLayout | LinearLayout | リニアレイアウト |
| - | android.widget.RelativeLayout | RelativeLayout | 相対レイアウト |
| - | android.widget.FrameLayout | FrameLayout | フレームレイアウト |
| - | androidx.constraintlayout.widget.ConstraintLayout | ConstraintLayout | 制約レイアウト |
| - | androidx.coordinatorlayout.widget.CoordinatorLayout | CoordinatorLayout | コーディネーターレイアウト |
| - | androidx.viewpager.widget.ViewPager | ViewPager | ビューページャー |
| - | androidx.viewpager2.widget.ViewPager2 | ViewPager2 | ビューページャー2 |
| - | androidx.core.widget.NestedScrollView | NestedScrollView | ネストスクロールビュー |
| - | com.google.android.material.button.MaterialButton | MaterialButton | マテリアルボタン |
| - | com.google.android.material.textfield.TextInputEditText | TextInputEditText | マテリアル入力 |
| - | com.google.android.material.floatingactionbutton.FloatingActionButton | FloatingActionButton | FAB |
| - | com.google.android.material.bottomnavigation.BottomNavigationView | BottomNavigationView | ボトムナビ |
| - | com.google.android.material.tabs.TabLayout | TabLayout | タブレイアウト |

**注**: `-` は該当プラットフォームに対応する要素がないことを示します。

## スクロール可能な要素

以下の要素タイプはスクロール可能として検出され、`[scrollable]` マーカーが付与されます。

| agent-mobile 表示名 | iOS | Android |
|--------------------|-----|---------|
| ScrollView | ✓ | ✓ |
| HorizontalScrollView | - | ✓ |
| Table / ListView | ✓ | ✓ |
| CollectionView / RecyclerView | ✓ | ✓ |
| WebView | ✓ | ✓ |
| List | ✓ | - |
| Grid | ✓ | - |
| GridView | - | ✓ |
| TextEditor | ✓ | - |
| ViewPager | - | ✓ |
| ViewPager2 | - | ✓ |
| NestedScrollView | - | ✓ |
| LazyColumn (Compose) | - | ✓ |
| LazyRow (Compose) | - | ✓ |
