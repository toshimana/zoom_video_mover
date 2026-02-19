# 開発ツール

## 📁 フォルダ構成

### plantuml/
PlantUML図生成ツール一式
- `plantuml.jar` - PlantUML実行ファイル
- `package.json` - Node.js依存関係設定
- `node_modules/` - Node.js依存関係

#### 使用方法
```bash
# PlantUML図の生成
cd tools/plantuml
java -jar plantuml.jar ../../docs/artifacts/design/diagrams/source/*.puml
```

### workspace/  
開発作業領域
- `drafts/` - 草稿・アイデア
- `experiments/` - 実験的コード
- `notes/` - 開発メモ
- `references/` - 参考資料
- `test_data/` - テストデータ
- `test_output/` - テスト出力結果
- `tests/` - 実験的テストコード

#### 使用方法
自由に作業ファイルを配置してください。
`.gitignore`で除外される一時ファイルの置き場としても利用可能です。

## 🚀 セットアップ

### PlantUML環境
```bash
cd tools/plantuml
npm install
```

### 図生成の自動化
```bash
# 監視モードで自動生成（要実装）
npm run watch
```