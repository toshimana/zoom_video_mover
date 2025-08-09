# ルートフォルダ整理戦略

## 現状分析

ルートフォルダには以下のような問題があります：

### 問題点
1. **文書の散在**: 仕様書類がルートに散乱（requirements.md, screen_specifications.md等）
2. **重複ファイル**: 同じ内容の文書が複数箇所に存在
3. **役割不明ファイル**: 用途が明確でないファイル群
4. **古いファイル**: 開発過程で作られた一時的なファイル
5. **混在構造**: 設定ファイル、文書、データファイルが混在

## 整理の基本方針

### 1. ファイル分類体系

```
📂 PROJECT_ROOT/
├── 🔧 Build & Config        # ビルド・設定ファイル
├── 📖 Documentation         # 主要文書（最小限）
├── 💻 Source Code          # ソースコード
├── 🧪 Tests                # テストコード・データ
├── 📊 Project Data         # プロジェクト管理データ
├── 🗃️ Archive/Legacy       # 廃止・旧バージョン
└── 🛠️ Tools & Scripts      # 開発ツール
```

### 2. ルートレベルファイルの原則

**ルートに置くべきファイル**:
- ✅ ビルド・設定ファイル（必須）
- ✅ メイン文書（README.md, CLAUDE.md等）
- ✅ ライセンス・法的文書

**ルートから除外すべきファイル**:
- ❌ 詳細仕様書（docs/配下へ）
- ❌ データファイル（data/配下へ）
- ❌ 一時ファイル（temp/配下へ）
- ❌ アーカイブ（archive/配下へ）

## ファイル分類と移動計画

### 🔧 Build & Config（ルートに残す）
```
Cargo.toml          # Rustプロジェクト設定
Cargo.lock          # 依存関係ロック
build.rs            # ビルドスクリプト
package.json        # Node.js依存関係（PlantUML用）
package-lock.json   # Node.jsロック
```

### 📖 Documentation（ルートに残す - 最小限）
```
README.md           # プロジェクト概要
CLAUDE.md           # Claude向け指示書
PROJECT_FEATURES.md # 機能概要（簡潔版）
QUICKSTART.md       # クイックスタート
```

### 💻 Source Code（ルートに残す）
```
src/                # ソースコード
```

### 🧪 Tests（ルートに残す）
```
tests/              # テストコード
```

### 📊 Project Data（新規作成 - data/）
```
data/
├── csv/                          # CSVデータ
│   ├── implementation_progress.csv
│   ├── traceability_matrix.csv
│   ├── traceability_relationship_matrix.csv
│   └── README.md
├── specifications/               # 詳細仕様書
│   ├── function_specifications.md
│   ├── operation_specifications.md
│   ├── screen_specifications.md
│   ├── zoom_api_specifications.md
│   ├── requirements.md
│   └── README.md
└── models/                      # モデル・図表
    ├── rdra_models.md
    └── README.md
```

### 🗃️ Archive/Legacy（新規作成 - archive/）
```
archive/
├── old_versions/                # 旧バージョンファイル
├── deprecated/                  # 廃止予定
│   ├── ARCHITECTURE.md          # → docs/artifacts/design/へ統合
│   ├── DEVELOPMENT_CHECKLIST.md # → docs/へ移動
│   ├── RUST_DEVELOPMENT.md      # → docs/policies/へ移動
│   └── POLICY_REUSE_GUIDE.md    # → docs/policies/へ移動
└── migration_notes/             # 移行メモ
    └── 2025-08-reorganization.md
```

### 🛠️ Tools & Scripts（新規作成 - tools/）
```
tools/
├── plantuml/                    # PlantUML関連
│   ├── plantuml.jar            # （3rdpartyから移動）
│   └── diagrams/               # （plantumlフォルダから移動）
├── temp_workspace/             # 作業領域
└── node_modules/               # Node.js依存関係
```

### 🗂️ その他整理対象
```
policies-template/              # → docs/templates/へ移動
traceability_matrix.md          # → data/csv/README.mdに統合
human_judgment_guidelines.md    # → 重複削除（docs/policies/に存在）
分析.xmind                      # → archive/analysis/へ移動
```

## 推奨ルート構造

### Before（現在）- 28個のファイル・フォルダ
```
PROJECT_ROOT/
├── ARCHITECTURE.md              # 散在文書
├── CLAUDE.md                   # メイン文書
├── Cargo.lock                  # ビルド設定
├── Cargo.toml                  # ビルド設定
├── DEVELOPMENT_CHECKLIST.md    # 散在文書
├── POLICY_REUSE_GUIDE.md       # 散在文書
├── PROJECT_FEATURES.md         # メイン文書
├── QUICKSTART.md               # メイン文書
├── README.md                   # メイン文書
├── RUST_DEVELOPMENT.md         # 散在文書
├── build.rs                    # ビルドスクリプト
├── docs/                       # 文書群
├── function_specifications.md   # 散在仕様書
├── human_judgment_guidelines.md # 重複ファイル
├── implementation_progress.csv  # データファイル
├── node_modules/               # ツール依存関係
├── operation_specifications.md  # 散在仕様書
├── package-lock.json           # ツール設定
├── package.json                # ツール設定
├── plantuml/                   # ツール関連
├── policies-template/          # テンプレート
├── rdra_models.md              # モデルファイル
├── requirements.md             # 散在仕様書
├── screen_specifications.md     # 散在仕様書
├── src/                        # ソースコード
├── temp_workspace/             # 作業領域
├── tests/                      # テストコード
├── traceability_matrix.csv     # データファイル
├── traceability_matrix.md      # データ説明
├── traceability_relationship_matrix.csv # データファイル
├── traceability_relationship_matrix.md  # データ説明
├── zoom_api_specifications.md   # 散在仕様書
└── 分析.xmind                   # 分析ファイル
```

### After（推奨）- 12個のファイル・フォルダ
```
PROJECT_ROOT/
├── README.md                   # プロジェクト概要
├── CLAUDE.md                   # Claude向け指示
├── PROJECT_FEATURES.md         # 機能概要（簡潔版）
├── QUICKSTART.md               # クイックスタート
├── Cargo.toml                  # Rustプロジェクト設定
├── Cargo.lock                  # 依存関係ロック
├── build.rs                    # ビルドスクリプト
├── package.json                # PlantUML用Node設定
├── src/                        # ソースコード
├── tests/                      # テストコード
├── docs/                       # 体系化された文書
└── data/                       # プロジェクトデータ
    ├── csv/                    # データファイル群
    ├── specifications/         # 詳細仕様書群
    └── models/                 # モデル・図表
```

## 整理手順

### Phase 1: データファイル整理
1. `data/csv/` フォルダ作成
2. CSV・データファイル群を移動
3. 対応するMDファイルを統合・整理

### Phase 2: 仕様書整理  
1. `data/specifications/` フォルダ作成
2. 散在する仕様書を移動
3. 重複・古いファイルを特定・削除

### Phase 3: ツール・作業ファイル整理
1. `tools/` フォルダ作成
2. plantuml、temp_workspace等を移動
3. node_modules整理

### Phase 4: アーカイブ整理
1. `archive/` フォルダ作成
2. 非推奨・古いファイルを移動
3. 重複ファイル削除

### Phase 5: テンプレート整理
1. `docs/templates/` フォルダ作成
2. policies-template移動・整理

### Phase 6: ルートクリーンアップ
1. 不要ファイル削除
2. 残すべきファイルの最終確認
3. 新構造のREADME更新

## 判断基準

### ルートに残すファイルの基準
1. **ビルド必須**: Cargo.toml, package.json等
2. **初見必読**: README.md, QUICKSTART.md等
3. **ツール設定**: build.rs, 設定ファイル
4. **標準構造**: src/, tests/, docs/

### 移動対象ファイルの基準
1. **詳細仕様**: 具体的な実装仕様書
2. **データファイル**: CSV、分析データ等
3. **作業ファイル**: 一時的、実験的ファイル
4. **アーカイブ**: 古い、重複ファイル

### 削除対象ファイルの基準
1. **重複ファイル**: 同内容で複数箇所に存在
2. **空ファイル**: 内容のないファイル
3. **破損ファイル**: 読み取り不可ファイル
4. **テンポラリ**: .tmp, .bak等一時ファイル

## メリット

1. **視認性向上**: ルートが整理され、重要ファイルが見つけやすい
2. **保守性向上**: 役割別整理で管理が容易
3. **新規参加者支援**: 構造が明確で理解しやすい
4. **ビルド効率化**: 不要ファイルによる干渉を排除
5. **バージョン管理**: 履歴が整理され、差分が明確

## 注意事項

1. **段階的実施**: 一度に全て変更せず、段階的に実施
2. **バックアップ**: 重要ファイルは移動前にバックアップ
3. **依存関係確認**: 移動前に他ファイルからの参照をチェック
4. **ツール設定更新**: パス変更に伴うツール設定の更新
5. **文書更新**: 新構造に合わせてREADME等を更新

この戦略により、ルートフォルダがスッキリと整理され、プロジェクトの保守性と可読性が大幅に向上します。