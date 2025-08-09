# ルートフォルダドキュメント分析

## 現在のルートドキュメント分析

### ルートに残すべきドキュメント（必須最小限）

#### 1. プロジェクト必須文書
- ✅ **README.md** - プロジェクト概要（必須・ルート維持）
- ✅ **CLAUDE.md** - AI開発指示書（必須・ルート維持）

#### 2. 新規参加者向け簡潔文書
- ✅ **QUICKSTART.md** - クイックスタート（簡潔版であれば維持）

### docs階層に移動すべきドキュメント

#### Level 1: 汎用ポリシー層への移動対象

**1. DEVELOPMENT_CHECKLIST.md**
- 現在: ルート
- 移動先: `docs/policies/universal/development_checklist.md`
- 理由: 開発プロセスの汎用ガイドライン

**2. human_judgment_guidelines.md**
- 現在: ルート（重複ファイル）
- 処理: 削除（既に`docs/policies/universal/`に存在）
- 理由: 完全に重複している

#### Level 2: 技術固有ポリシー層への移動対象

**3. RUST_DEVELOPMENT.md**
- 現在: ルート
- 移動先: `docs/policies/technology-specific/rust/rust_development_guide.md`
- 理由: Rust固有の開発ガイド

#### Level 3: プロジェクト汎用ポリシー層への移動対象

**4. POLICY_REUSE_GUIDE.md**
- 現在: ルート
- 移動先: `docs/policies/project-generic/zoom-video-mover/policy_reuse_guide.md`
- 理由: プロジェクト固有のポリシー運用ガイド

**5. PROJECT_FEATURES.md**
- 現在: ルート
- 判定: **条件付き移動** or **簡潔版作成**
- 移動先: `docs/artifacts/requirements/project_features_detailed.md`
- ルート版: 簡潔な概要版を新規作成し維持

#### 成果物層への移動対象

**6. ARCHITECTURE.md**
- 現在: ルート
- 移動先: `docs/artifacts/design/system_architecture.md`
- 理由: システム設計成果物（重複あり要統合）

#### データ・仕様書の移動対象

**7. requirements.md**
- 現在: ルート
- 移動先: `docs/artifacts/requirements/system_requirements.md`
- 理由: 詳細要件定義成果物

**8. function_specifications.md**
- 現在: ルート
- 移動先: `docs/artifacts/requirements/function_specifications.md`
- 理由: 機能仕様成果物

**9. operation_specifications.md**
- 現在: ルート
- 移動先: `docs/artifacts/requirements/operation_specifications.md`
- 理由: 操作仕様成果物

**10. screen_specifications.md**
- 現在: ルート
- 移動先: `docs/artifacts/design/ui_specifications.md`
- 理由: UI設計成果物

**11. zoom_api_specifications.md**
- 現在: ルート
- 移動先: `docs/artifacts/design/api_specifications.md`
- 理由: API設計成果物

**12. rdra_models.md**
- 現在: ルート
- 移動先: `docs/artifacts/requirements/rdra_analysis_models.md`
- 理由: RDRA分析成果物

#### データファイル・トレーサビリティ

**13. traceability_matrix.csv/.md**
- 現在: ルート
- 移動先: `docs/artifacts/requirements/traceability/`
- 理由: 要件管理成果物

**14. implementation_progress.csv**
- 現在: ルート
- 移動先: `docs/artifacts/implementation/progress_tracking.csv`
- 理由: 実装進捗管理データ

## 移動後のルート構造（推奨）

### Before: 28個のファイル・フォルダ
### After: 8個の厳選されたファイル・フォルダ

```
PROJECT_ROOT/
├── README.md                   # プロジェクト概要（必須）
├── CLAUDE.md                   # AI開発指示書（必須）
├── QUICKSTART.md               # クイックスタート（簡潔版）
├── Cargo.toml                  # Rustビルド設定
├── Cargo.lock                  # 依存関係ロック
├── build.rs                    # ビルドスクリプト  
├── src/                        # ソースコード
├── tests/                      # テストコード
└── docs/                       # 体系化されたドキュメント
    ├── policies/               # ポリシー層
    ├── rules/                  # ルール層  
    └── artifacts/              # 成果物層
```

## 具体的な移動計画

### Phase 1: 重複ファイル削除
```bash
rm human_judgment_guidelines.md  # 既にdocs/policies/universal/に存在
```

### Phase 2: ポリシー層への移動
```bash
mv DEVELOPMENT_CHECKLIST.md docs/policies/universal/development_checklist.md
mv RUST_DEVELOPMENT.md docs/policies/technology-specific/rust/rust_development_guide.md  
mv POLICY_REUSE_GUIDE.md docs/policies/project-generic/zoom-video-mover/policy_reuse_guide.md
```

### Phase 3: 成果物層への移動
```bash
# 要件定義成果物
mv requirements.md docs/artifacts/requirements/system_requirements.md
mv function_specifications.md docs/artifacts/requirements/function_specifications.md
mv operation_specifications.md docs/artifacts/requirements/operation_specifications.md
mv rdra_models.md docs/artifacts/requirements/rdra_analysis_models.md

# 設計成果物
mv ARCHITECTURE.md docs/artifacts/design/system_architecture.md
mv screen_specifications.md docs/artifacts/design/ui_specifications.md  
mv zoom_api_specifications.md docs/artifacts/design/api_specifications.md

# 実装成果物
mv implementation_progress.csv docs/artifacts/implementation/progress_tracking.csv
```

### Phase 4: トレーサビリティ整理
```bash
mkdir -p docs/artifacts/requirements/traceability/
mv traceability_matrix.csv docs/artifacts/requirements/traceability/
mv traceability_matrix.md docs/artifacts/requirements/traceability/
mv traceability_relationship_matrix.csv docs/artifacts/requirements/traceability/
mv traceability_relationship_matrix.md docs/artifacts/requirements/traceability/
```

### Phase 5: PROJECT_FEATURES.mdの処理
```bash
# 詳細版を成果物へ移動
mv PROJECT_FEATURES.md docs/artifacts/requirements/project_features_detailed.md

# 簡潔版を新規作成してルートに配置
# （主要機能の概要のみ1-2ページ程度）
```

## 判断基準の詳細

### ルートに残す基準
1. **新規参加者が最初に読むべき** - README.md, CLAUDE.md
2. **プロジェクト全体を1分で理解** - QUICKSTART.md（簡潔版）
3. **ビルドに直接必要** - Cargo.toml, build.rs
4. **標準的なフォルダ構造** - src/, tests/, docs/

### docs階層に移動する基準  
1. **詳細な仕様・設計文書** - 実装者向けの詳細情報
2. **プロセス・ガイドライン** - 開発手法・規約
3. **分析・管理データ** - トレーサビリティ、進捗管理
4. **技術固有の情報** - 特定技術に関する詳細

## メリット

1. **認知負荷軽減**: ルートで迷わない、重要ファイルが明確
2. **階層一貫性**: ドキュメント体系化フレームワークとの統合
3. **保守性向上**: 分類が明確で更新・管理が容易
4. **新規参加者体験**: プロジェクト理解の段階的アプローチ
5. **検索効率**: ファイルの場所が予測しやすい

この移動により、ルートフォルダは「プロジェクトの玄関」として機能し、詳細情報は適切に分類されたdocs階層で管理されます。