# PlantUML RDRA モデル図

このディレクトリには、RDRA（Requirements Definition and Requirements Architecture）に従って作成されたPlantUMLファイルが格納されています。

## 図一覧

| ファイル名 | 図の種類 | 説明 |
|-----------|---------|------|
| `01_system_value_hierarchy.puml` | システム価値体系図 | ビジネス価値→目的→システム機能の階層構造 |
| `02_system_context.puml` | システムコンテキスト図 | システムと外部環境の関係性・境界 |
| `03_business_flow.puml` | ビジネスフロー図 | 録画ダウンロード業務プロセス |
| `04_business_usecase.puml` | ビジネスユースケース図 | アクター別の利用シナリオ |
| `05_requirements_specification.puml` | 要求仕様書 | 機能要求と非機能要求の整理 |
| `06_system_architecture.puml` | システム方式設計図 | アーキテクチャとコンポーネント構成 |

## PlantUMLファイルの表示方法

### 1. オンラインエディタ
- [PlantUML Online Server](http://www.plantuml.com/plantuml/uml/)
- ファイル内容をコピー&ペーストして表示

### 2. VSCodeプラグイン
```bash
# PlantUML拡張機能をインストール
code --install-extension jebbs.plantuml
```

### 3. コマンドライン
```bash
# PlantUMLをインストール（Java必須）
# Windows: choco install plantuml
# macOS: brew install plantuml
# Ubuntu: apt-get install plantuml

# PNG画像として出力
plantuml -tpng *.puml

# SVG画像として出力
plantuml -tsvg *.puml
```

### 4. 一括画像生成
```bash
# 全ファイルをPNG形式で出力
plantuml -tpng plantuml/*.puml

# 出力先ディレクトリ指定
plantuml -tpng -o ../images plantuml/*.puml
```

## 図の依存関係

```
システム価値体系図
    ↓ (価値の具体化)
システムコンテキスト図
    ↓ (境界の明確化)
ビジネスフロー図
    ↓ (プロセスの詳細化)
ビジネスユースケース図
    ↓ (機能の抽出)
要求仕様書
    ↓ (技術的実現)
システム方式設計図
```

## RDRAモデルの効果

1. **要件の可視化**: ステークホルダー間での認識共有
2. **要件の構造化**: 価値から実装まで一貫した整理
3. **要件の検証**: 各レベルでの妥当性確認
4. **変更管理**: 影響範囲の特定と変更容易性
5. **開発指針**: アーキテクチャ設計の根拠明確化

## 更新履歴

- 2025-07-27: 初版作成（全6図完成）
- RDRAメソドロジーに基づく体系的な要件定義モデル構築完了

## 関連ドキュメント

- `../requirements.md`: 詳細要件仕様書
- `../rdra_models.md`: RDRA手法による要件整理
- `../CLAUDE.md`: プロジェクト全体の技術仕様