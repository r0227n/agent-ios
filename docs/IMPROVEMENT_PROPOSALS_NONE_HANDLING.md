# None 処理パターンの改善提案

## 概要

このドキュメントは、コードベース内の `None` 処理パターンの調査結果に基づく改善提案です。

## 現状の問題点

### 1. 一貫性の欠如

現在、同じような処理に対して異なるパターンが使用されています：

```rust
// パターン1: .map().unwrap_or()
elem.label.as_ref().map(|l| l == text).unwrap_or(false)

// パターン2: .as_deref().unwrap_or()
element.label.as_deref().unwrap_or("-")

// パターン3: .unwrap_or_default()
element.label.clone().unwrap_or_default()
```

### 2. 暗黙的なフォールバック値

多くの箇所で、フォールバック値の意味が明確ではありません：

```rust
// src/core/find.rs:504-508
let clear_len = element
    .value
    .as_ref()
    .map(|v| v.chars().count())
    .unwrap_or(50);  // なぜ50？コメントなし
```

### 3. エラーハンドリングの曖昧さ

`None` が正常なケースなのかエラーなのか不明確：

```rust
// src/core/ref_resolver.rs:120-128
fn parse_coords(s: &str) -> Option<(f64, f64)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return None;  // これはエラー？正常？
    }
    // ...
}
```

## 改善提案

### 提案1: カスタムエラー型の導入 🔴 優先度: 高

**現状の問題:**
`None` を返すだけでは、なぜ失敗したのか（無効なフォーマット、要素が見つからない、等）が不明確です。

**改善案:**

```rust
// src/core/errors.rs (新規作成)
#[derive(Debug, thiserror::Error)]
pub enum ElementError {
    #[error("Invalid coordinate format: expected 'x,y' but got '{0}'")]
    InvalidCoordinateFormat(String),

    #[error("Invalid swipe format: expected 'x1,y1,x2,y2' but got '{0}'")]
    InvalidSwipeFormat(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

// 使用例: src/core/ref_resolver.rs
fn parse_coords(s: &str) -> Result<(f64, f64), ElementError> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(ElementError::InvalidCoordinateFormat(s.to_string()));
    }

    let x = parts[0].trim().parse()
        .map_err(|_| ElementError::ParseError(format!("Invalid x coordinate: {}", parts[0])))?;
    let y = parts[1].trim().parse()
        .map_err(|_| ElementError::ParseError(format!("Invalid y coordinate: {}", parts[1])))?;

    Ok((x, y))
}
```

**効果:**
- エラーメッセージが具体的になる
- デバッグが容易になる
- AI エージェントがエラーを理解しやすくなる

---

### 提案2: 定数化とドキュメント化 🟡 優先度: 中

**現状の問題:**
マジックナンバーが散在しています。

**改善案:**

```rust
// src/core/find.rs
/// テキストフィールドの最大文字数の推定値
/// value が None の場合、この値を使用して全文削除を試みる
const DEFAULT_MAX_TEXT_LENGTH: usize = 50;

/// Execute fill (tap + clear + type)
async fn execute_fill(
    platform: Platform,
    udid: Option<&str>,
    x: f64,
    y: f64,
    text: &str,
    clear_len: usize,
) -> CommandResult {
    // ...
    let clear_len = element
        .value
        .as_ref()
        .map(|v| v.chars().count())
        .unwrap_or(DEFAULT_MAX_TEXT_LENGTH);
    // ...
}
```

**効果:**
- 意図が明確になる
- 調整が容易になる
- ドキュメントとして機能する

---

### 提案3: ヘルパー関数の導入 🟡 優先度: 中

**現状の問題:**
同じパターンが繰り返し出現しています。

**改善案:**

```rust
// src/snapshot/helpers.rs (新規作成)

/// Check if element matches text in label or value
pub fn element_contains_text(elem: &SnapshotElement, text: &str, case_sensitive: bool) -> bool {
    let text = if case_sensitive {
        text.to_string()
    } else {
        text.to_lowercase()
    };

    let matches_label = elem.label.as_ref()
        .map(|l| {
            let label = if case_sensitive { l.clone() } else { l.to_lowercase() };
            label.contains(&text)
        })
        .unwrap_or(false);

    let matches_value = elem.value.as_ref()
        .map(|v| {
            let value = if case_sensitive { v.clone() } else { v.to_lowercase() };
            value.contains(&text)
        })
        .unwrap_or(false);

    matches_label || matches_value
}

/// Get display text for element (label or value or placeholder)
pub fn element_display_text(elem: &SnapshotElement) -> &str {
    elem.label.as_deref()
        .or(elem.value.as_deref())
        .or(elem.placeholder.as_deref())
        .unwrap_or("-")
}

// 使用例: src/core/find.rs
fn matches_locator(elem: &SnapshotElement, locator: &FindLocator) -> bool {
    match locator {
        FindLocator::Text { text, exact, .. } => {
            element_contains_text(elem, text, *exact)
        }
        // ...
    }
}
```

**効果:**
- コードの重複削減
- テストが容易になる
- 一貫性の向上

---

### 提案4: より明示的な Option 処理 🟢 優先度: 低

**現状の問題:**
`unwrap_or()` の多用で、デフォルト値の意味が不明確です。

**改善案:**

```rust
// Before
let label_part = elem.label.as_ref()
    .map(|l| format!(" \"{}\"", l))
    .unwrap_or_default();

// After: match 式で明示的に
let label_part = match &elem.label {
    Some(label) => format!(" \"{}\"", label),
    None => String::new(),  // 明示的に「ラベルなし」を示す
};

// または: if let で読みやすく
let label_part = if let Some(label) = &elem.label {
    format!(" \"{}\"", label)
} else {
    String::new()
};
```

**効果:**
- 意図が明確になる
- レビュー時に理解しやすい
- 初心者にも優しい

---

### 提案5: 検証ロジックの分離 🟡 優先度: 中

**現状の問題:**
is_none() チェックが複雑な条件式に埋もれています。

**改善案:**

```rust
// src/snapshot/ref_generator.rs:120-122
// Before
pub fn is_empty_structure(element: &SnapshotElement) -> bool {
    element.label.is_none() && element.value.is_none() && !element.is_interactive
}

// After: より詳細なヘルパー関数
impl SnapshotElement {
    /// 要素が表示可能なテキストを持っているか
    pub fn has_display_text(&self) -> bool {
        self.label.is_some() || self.value.is_some() || self.placeholder.is_some()
    }

    /// 要素がユーザーに見えるコンテンツを持っているか
    pub fn has_visible_content(&self) -> bool {
        self.has_display_text() || self.is_interactive
    }

    /// 要素が空の構造体か（レンダリング不要）
    pub fn is_empty_structure(&self) -> bool {
        !self.has_visible_content()
    }
}

// 使用例
if element.has_display_text() {
    // ラベルまたは値を表示
}
```

**効果:**
- 可読性の向上
- 再利用性の向上
- ビジネスロジックが明確になる

---

## 実装優先順位

### Phase 1: 基礎改善 (1-2日)
1. ✅ **提案2: 定数化とドキュメント化**
   - マジックナンバーを定数化
   - 既存コードへの影響が最小限

2. ✅ **提案5: 検証ロジックの分離**
   - ヘルパーメソッドを追加
   - 既存コードとの互換性を保ちつつ改善

### Phase 2: 構造改善 (2-3日)
3. ✅ **提案3: ヘルパー関数の導入**
   - 重複コードをリファクタリング
   - テストを追加

### Phase 3: エラーハンドリング改善 (3-5日)
4. ✅ **提案1: カスタムエラー型の導入**
   - `thiserror` クレートの依存追加
   - 段階的に既存コードを移行

### Phase 4: コードスタイル統一 (継続的)
5. ✅ **提案4: より明示的な Option 処理**
   - 新規コードから適用
   - 既存コードは必要に応じて更新

---

## 影響範囲の評価

### 破壊的変更
- **提案1**: `parse_coords()` などの戻り値が `Option<T>` から `Result<T, E>` に変更
  - 影響ファイル: `ref_resolver.rs`, `swipe.rs`
  - 移行コスト: 中程度（呼び出し側の修正が必要）

### 非破壊的変更
- **提案2, 3, 5**: 新しいヘルパー関数/定数の追加
  - 影響: なし（既存コードはそのまま動作）
  - 移行コスト: 低（段階的に置き換え可能）

---

## 測定可能な成果

### Before (現状)
```rust
// 15行のテキストマッチングロジック (find.rs:369-388)
FindLocator::Text { text, exact, .. } => {
    let text_lower = text.to_lowercase();
    if *exact {
        elem.label
            .as_ref()
            .map(|l| l.to_lowercase() == text_lower)
            .unwrap_or(false)
            || elem.value.as_ref()
                .map(|v| v.to_lowercase() == text_lower)
                .unwrap_or(false)
    } else {
        elem.label.as_ref()
            .map(|l| l.to_lowercase().contains(&text_lower))
            .unwrap_or(false)
            || elem.value.as_ref()
                .map(|v| v.to_lowercase().contains(&text_lower))
                .unwrap_or(false)
    }
}
```

### After (改善後)
```rust
// 3行のシンプルなロジック
FindLocator::Text { text, exact, .. } => {
    element_contains_text(elem, text, !exact)
}
```

**削減率**: 80% (15行 → 3行)

---

## 次のステップ

1. **このドキュメントのレビュー**
   - プロジェクトメンバーと改善提案を議論

2. **Phase 1 の実装開始**
   - 定数化とヘルパーメソッドの追加
   - PR を作成してレビュー

3. **段階的なロールアウト**
   - 新規コードから適用
   - 既存コードは issue を作成して計画的に改善

4. **効果測定**
   - コードレビュー時間の短縮
   - バグ発生率の低下
   - AI エージェントの理解精度向上

---

## 参考資料

- [Rust Error Handling Best Practices](https://doc.rust-lang.org/book/ch09-00-error-handling.html)
- [thiserror crate](https://docs.rs/thiserror/)
- [Option and Result - Rust by Example](https://doc.rust-lang.org/rust-by-example/error/option_unwrap.html)

---

**作成日**: 2026-01-24
**対象バージョン**: agent-mobile v0.x
**ステータス**: 提案中 (Proposed)
