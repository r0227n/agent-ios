---
name: mobile-e2e
description: iOS/Android で agent-mobile CLI の E2E テストを実施し、Markdown レポートを作成するスキル。
user-invocable: true
argument-hint: <ios|android> [--quick] [--checklist <path>]
---

# Mobile E2E Testing Skill

## Overview

このスキルは `agent-mobile` CLI の E2E テストを実施し、テスト結果を Markdown レポートとして作成します。

## Arguments

| 引数 | 説明 |
|------|------|
| `ios` | iOS E2E テストを実施 |
| `android` | Android E2E テストを実施 |
| `--quick` | 軽量版チェックリストを使用 (デフォルト: 完全版) |
| `--checklist <path>` | カスタムチェックリストを指定 |

## Workflow

### Phase 1: 環境準備

1. **デバイス確認**
   ```bash
   agent-mobile device list -p <platform>
   ```
   - 利用可能なデバイス/エミュレータを確認
   - Booted 状態のデバイスがない場合は起動を促す

2. **セッション作成**
   ```bash
   agent-mobile session create --session e2e-test
   ```
   - テスト用セッションを作成
   - 既存セッションがあれば再利用可能

3. **テストアプリ起動** (任意)
   - 必要に応じてテスト対象アプリを起動
   - iOS: Safari (`com.apple.mobilesafari`) など
   - Android: Settings (`com.android.settings`) など

### Phase 2: チェックリスト読み込み

1. **チェックリストの選択**
   - `--quick` 指定時: `references/<platform>-quick-checklist.md`
   - デフォルト: `references/<platform>-full-checklist.md`
   - `--checklist` 指定時: 指定されたパス

2. **チェックリストの読み込み**
   - Read ツールでチェックリストファイルを読み込む
   - テスト項目を把握

### Phase 3: E2E テスト実施

1. **各テスト項目を順番に実行**
   - コマンドを実行
   - 結果を記録 (成功/失敗/スキップ)
   - エラーメッセージがあれば記録

2. **スクリーンショット取得** (重要な確認時)
   ```bash
   agent-mobile screenshot -o <output_path>
   ```

3. **テスト間の状態リセット** (必要に応じて)
   - アプリ再起動
   - ホーム画面に戻る など

### Phase 4: レポート作成

1. **レポートファイル作成**
   - `references/report-format.md` のフォーマットに従う
   - ファイル名: `e2e-report-<platform>-<timestamp>.md`

2. **レポート内容**
   - テスト環境情報
   - 成功/失敗/スキップの集計
   - 各テスト項目の結果詳細
   - 発見事項・課題
   - 次のステップ

3. **セッション削除** (任意)
   ```bash
   agent-mobile session destroy --session e2e-test
   ```

## Checklist Files

### iOS
- **完全版**: `references/ios-full-checklist.md` (~400項目)
- **軽量版**: `references/ios-quick-checklist.md` (~50項目)

### Android
- **完全版**: `references/android-full-checklist.md`
- **軽量版**: `references/android-quick-checklist.md`

## Usage Examples

### iOS 完全版テスト
```
/mobile-e2e ios
```

### iOS 軽量版テスト
```
/mobile-e2e ios --quick
```

### Android 完全版テスト
```
/mobile-e2e android
```

### カスタムチェックリスト使用
```
/mobile-e2e ios --checklist ./my-checklist.md
```

## Important Notes

1. **実機確認の重要性**: テストは必ず実機/シミュレータ/エミュレータで確認する
2. **要素参照の動的変更**: `snapshot` を取得するたびに要素参照 (@eN) が変わる可能性がある
3. **セッション管理**: セッションを使用することで複数コマンドを効率的に実行できる
4. **JSON 出力**: `--format json` オプションで解析しやすい出力を取得可能

## Reference Files

- [iOS Full Checklist](references/ios-full-checklist.md)
- [iOS Quick Checklist](references/ios-quick-checklist.md)
- [Android Full Checklist](references/android-full-checklist.md)
- [Android Quick Checklist](references/android-quick-checklist.md)
- [Report Format](references/report-format.md)
