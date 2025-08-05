# 優先対応アクションアイテム - Zoom Video Mover

## 🔴 Phase 0: セキュリティ緊急対応（1週間以内）

### 1. トークン暗号化実装
```rust
// 実装タスク
- [ ] AES-256-GCM cipher実装 (src/components/auth/crypto_utils.rs)
- [ ] SecureStorageトレイト実装 (src/components/auth/secure_storage.rs)
- [ ] Windows DPAPI統合
- [ ] メモリゼロ化処理追加
- [ ] 暗号化キー管理実装
```

**影響箇所**:
- `src/components/auth.rs`
- `src/components/config.rs`
- 新規ファイル作成必要

### 2. PKCE完全実装
```rust
// 実装タスク
- [ ] OAuthConfigにuse_pkce設定追加
- [ ] exchange_code_for_tokenでのstate検証
- [ ] PKCE verifier保存・検証ロジック
- [ ] エラーハンドリング強化
```

**影響箇所**:
- `src/components/auth.rs`: generate_auth_url(), exchange_code()
- `src/components/config.rs`: OAuthConfig構造体

### 3. 入力検証フレームワーク
```rust
// 実装タスク
- [ ] 包括的バリデーター作成
- [ ] SQLインジェクション対策
- [ ] パストラバーサル防止
- [ ] XSS防御
- [ ] URL検証強化
```

**影響箇所**:
- 全コンポーネントの入力処理
- 新規ファイル: `src/security/input_validator.rs`

### 4. セキュアメモリ管理
```rust
// 実装タスク
- [ ] secure_zero_memory関数実装
- [ ] 機密データ構造体にDrop実装
- [ ] zeroizeクレート統合検討
```

**影響箇所**:
- `src/components/auth.rs`
- `src/lib.rs`

## 🟡 Phase 1: コア機能実装（1-4週間）

### 5. APIコンポーネント実装
```rust
// 実装タスク
- [ ] Zoom API クライアント基本実装
- [ ] レート制限処理
- [ ] ページネーション
- [ ] エラーハンドリング
- [ ] リトライロジック
```

**新規ファイル**:
- `src/components/api/client.rs`
- `src/components/api/rate_limiter.rs`
- `src/components/api/models.rs`

### 6. ダウンロードエンジン実装
```rust
// 実装タスク
- [ ] 並列ダウンロードマネージャー
- [ ] チャンク処理実装
- [ ] 進捗監視システム
- [ ] レジューム機能
- [ ] SHA-256検証
```

**新規ファイル**:
- `src/components/download/parallel_engine.rs`
- `src/components/download/progress_monitor.rs`
- `src/components/download/chunk_processor.rs`

### 7. 認証コンポーネント強化
```rust
// 実装タスク
- [ ] AuthState列挙型実装
- [ ] 状態遷移管理
- [ ] トークン自動更新しきい値
- [ ] イベント通知システム
- [ ] UserInfo管理
```

**影響箇所**:
- `src/components/auth.rs`
- `src/components/auth/state_manager.rs` (新規)

## 🟢 Phase 2: UI・統合（4-6週間）

### 8. GUI実装
```rust
// 実装タスク
- [ ] eGUI基本レイアウト
- [ ] タブベースナビゲーション
- [ ] 進捗表示コンポーネント
- [ ] エラー通知システム
- [ ] 日本語フォント対応
```

### 9. 統合テスト
```rust
// 実装タスク
- [ ] エンドツーエンドテスト
- [ ] セキュリティテスト
- [ ] パフォーマンステスト
- [ ] Property-basedテスト拡充
```

## 📋 チェックリスト形式のタスク

### Week 1: セキュリティ基盤
- [ ] 月曜: AES-GCM実装開始
- [ ] 火曜: SecureStorage実装
- [ ] 水曜: PKCE修正・state検証
- [ ] 木曜: 入力検証フレームワーク
- [ ] 金曜: セキュアメモリ・テスト

### Week 2-3: API・ダウンロード
- [ ] APIクライアント基本構造
- [ ] レート制限実装
- [ ] 並列ダウンロードエンジン
- [ ] チャンク処理
- [ ] 進捗監視

### Week 4-5: 統合・UI
- [ ] コンポーネント統合
- [ ] GUI基本実装
- [ ] エラーハンドリング統合
- [ ] 統合テスト作成

## 🎯 成功基準

### セキュリティ
- [ ] すべてのトークンが暗号化保存される
- [ ] PKCE認証フローが正常動作
- [ ] 入力検証で既知の攻撃を防御
- [ ] メモリダンプに機密情報が残らない

### 機能
- [ ] 認証→API呼び出し→ダウンロードの基本フロー動作
- [ ] エラー時の適切な回復処理
- [ ] 進捗表示の正確性

### 品質
- [ ] テストカバレッジ 60%以上
- [ ] Property-basedテスト 50ケース以上
- [ ] セキュリティ監査パス

## 📞 エスカレーション

問題発生時の連絡先：
1. セキュリティ問題 → 即座にチームリーダーへ
2. 設計不明点 → 設計文書作成者へ確認
3. 実装ブロッカー → 週次ミーティングで議論

---

**作成日**: 2025-08-05  
**最終更新**: 2025-08-05  
**次回レビュー**: Week 1完了時