# フォルダクリーンアップ分析

## 発見された問題のあるフォルダ・構造

### 🗂️ 1. docs/03-development/ - 古い構造が残存

**問題**:
- `docs/03-development/` - 古いフォルダ番号体系
- `docs/03-development/policies/technology-specific/` - 空フォルダ
- `docs/03-development/guides/` - 統合されていないガイド

**現状**:
```
docs/
├── 03-development/              # ❌ 古いフォルダ番号体系
│   ├── README.md
│   ├── guides/
│   │   └── pfd_creation_guidelines.md
│   └── policies/
│       ├── README.md
│       ├── REORGANIZATION_PLAN.md
│       └── technology-specific/  # ❌ 空フォルダ
└── policies/                    # ✅ 新体系（正しい場所）
```

**解決策**:
- `docs/03-development/guides/` → `docs/guides/`
- `docs/03-development/policies/README.md` → 統合または移動
- 空の`technology-specific/`フォルダ削除
- `docs/03-development/`フォルダ削除

### 🗂️ 2. docs/rules/ - 空フォルダ群

**問題**:
- `docs/rules/`配下の全フォルダが空
- 階層構造は作成済みだが、コンテンツが未作成

**現状**:
```
docs/rules/
├── universal/                   # ❌ 空フォルダ
├── technology-specific/rust/    # ❌ 空フォルダ
├── project-generic/zoom-video-mover/ # ❌ 空フォルダ
└── project-specific/zoom-video-mover/ # ❌ 空フォルダ
```

**解決策**:
- 将来の拡張用として保持するか削除するかを判断
- README.mdを追加して用途を明確化
- または一旦削除して必要時に再作成

### 🗂️ 3. docs/99-archive/ - アーカイブの整理

**問題**:
- アーカイブ内容の整理が不十分
- 日付別フォルダ（2025-08/）が中途半端

**現状**:
```
docs/99-archive/
├── README.md
└── analysis/
    └── 2025-08/                 # ❓ 特定月のみ
        ├── comprehensive_*.md   # 大量の分析レポート
        └── ...
```

**解決策**:
- アーカイブ構造の統一
- 不要な古い分析レポートの整理

### 🗂️ 4. docs/templates/policies/ - 命名不統一

**問題**:
- `tech-specific/` vs `technology-specific/` の命名不統一
- `universal/` vs `project-generic/` の階層不統一

**現状**:
```
docs/templates/policies/
├── README.md
├── tech-specific/               # ❌ 短縮形
│   └── rust/
└── universal/                   # ❓ project-genericが無い
```

**正しい階層**:
```
docs/templates/policies/
├── universal/
├── technology-specific/         # ✅ 完全形に統一
├── project-generic/
└── project-specific/
```

## 🎯 総合的な再編計画

### Phase 1: 古い開発フォルダの統合
```bash
# ガイドを適切な場所に移動
mv docs/03-development/guides/ docs/guides/

# 不要ファイルの整理
mv docs/03-development/policies/README.md docs/policies/
mv docs/03-development/policies/REORGANIZATION_PLAN.md docs/

# 空フォルダ・古いフォルダの削除
rm -rf docs/03-development/
```

### Phase 2: 空のrulesフォルダの処理
**オプション A**: 削除
```bash
rm -rf docs/rules/
```

**オプション B**: README追加で将来用途明確化
```bash
# 各フォルダにREADME.md追加（将来拡張用）
```

### Phase 3: テンプレート階層の統一
```bash
# 命名統一
mv docs/templates/policies/tech-specific/ docs/templates/policies/technology-specific/

# 不足階層の追加
mkdir -p docs/templates/policies/project-generic/
mkdir -p docs/templates/policies/project-specific/
```

### Phase 4: アーカイブの整理
```bash
# 古い分析レポートの整理（必要に応じて）
# 日付フォルダ構造の統一
```

## 🏗️ 最終的な理想構造

```
docs/
├── README.md                    # メイン説明
├── INDEX.md                     # インデックス
├── guides/                      # 開発ガイド
│   └── pfd_creation_guidelines.md
├── policies/                    # ポリシー層
│   ├── universal/
│   ├── technology-specific/
│   ├── project-generic/
│   └── project-specific/
├── rules/                       # ルール層（将来用）
│   └── README.md               # 用途説明
├── templates/                   # テンプレート
│   └── policies/
│       ├── universal/
│       ├── technology-specific/
│       ├── project-generic/
│       └── project-specific/
├── artifacts/                   # 成果物
│   ├── requirements/
│   ├── design/
│   ├── implementation/
│   └── testing/
└── 99-archive/                 # アーカイブ
    └── analysis/
```

## 🎯 優先度

### 高優先度（即座に実行）
1. **docs/03-development/ の統合削除**
   - 古い構造の完全排除
   - ガイドの適切な配置

2. **空フォルダの削除**
   - `docs/03-development/policies/technology-specific/`
   - その他の空フォルダ

### 中優先度（近日中）
3. **テンプレート階層統一**
   - 命名規則の統一
   - 不足階層の補完

4. **rulesフォルダの処理**
   - 削除 or README追加

### 低優先度（時間があるとき）
5. **アーカイブ整理**
   - 古い分析レポートの精査
   - 構造の統一

## メリット

この再編により：
- ✅ フォルダ構造の完全統一
- ✅ 古い構造の完全排除  
- ✅ 命名規則の一貫性
- ✅ 空フォルダの排除
- ✅ 体系的なドキュメント管理

プロジェクトの保守性と可読性が大幅に向上します。