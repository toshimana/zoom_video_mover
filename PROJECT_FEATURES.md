# Zoom Video Mover

## 概要
ZoomクラウドレコーディングをローカルにダウンロードするRust GUIアプリケーション

## 主要機能
- **OAuth認証**: Zoom APIへの安全な接続
- **録画ダウンロード**: 動画・音声・チャット・トランスクリプトの一括取得
- **AI要約**: Zoom AI Companionによる会議要約の自動取得
- **並列処理**: 複数ファイルの効率的な同時ダウンロード

## プラットフォーム
- **Windows**: 日本語環境完全対応（主要サポート）
- **Mac/Linux**: 基本機能対応

## 技術スタック
- **言語**: Rust
- **GUI**: egui
- **非同期**: Tokio
- **認証**: OAuth 2.0

## クイックスタート
1. [QUICKSTART.md](QUICKSTART.md) を参照
2. OAuth設定 → 認証 → ダウンロード開始

## 詳細仕様
📄 **[詳細機能仕様](docs/artifacts/requirements/project_features_detailed.md)** - 完全な機能リスト・API仕様・制限事項

## 開発者向け
📖 **[ドキュメント](docs/)** - 設計文書・実装ガイド・テスト戦略