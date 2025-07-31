# Zoom API仕様書 - Zoom Video Mover プロジェクト用

本書は、Zoom Video Moverプロジェクトで使用するZoom APIの仕様をまとめたものです。

## 1. 概要

### Base URL
- **API Base URL**: `https://api.zoom.us/v2/`
- **OAuth Base URL**: `https://zoom.us/oauth/`
- **プロトコル**: HTTPS必須
- **認証**: OAuth 2.0またはServer-to-Server OAuth

### APIの分類
Zoom APIは以下のカテゴリに分類されます：
- **Light APIs**: 軽量なAPI（最も制限が緩い）
- **Medium APIs**: 中程度のAPI
- **Heavy APIs**: 重いAPI
- **Resource-intensive APIs**: リソース集約的API（最も制限が厳しい）

## 2. OAuth 2.0認証

### 2.1 認証フロー
1. **Authorization Code Grant**: ユーザー認証用
2. **Server-to-Server OAuth**: アカウント間認証用
3. **Device Authorization**: デバイス認証用

### 2.2 認証エンドポイント
```
POST https://zoom.us/oauth/token
```

#### パラメータ
| パラメータ | 型 | 必須 | 説明 |
|------------|----|----- |------|
| grant_type | string | ✓ | `authorization_code`, `refresh_token`, `client_credentials` |
| client_id | string | ✓ | OAuth アプリケーションの Client ID |
| client_secret | string | ✓ | OAuth アプリケーションの Client Secret |
| code | string | 条件付き | Authorization codeの場合必須 |
| redirect_uri | string | 条件付き | Authorization codeの場合必須 |
| refresh_token | string | 条件付き | Refresh tokenの場合必須 |

#### レスポンス例
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "bearer",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3599,
  "scope": "recording:read user:read meeting:read"
}
```

### 2.3 必要なスコープ
本プロジェクトで必要なOAuthスコープ：
- **`recording:read`**: 録画ファイルの読み取り（必須）
- **`user:read`**: ユーザー情報の読み取り（必須）
- **`meeting:read`**: 会議情報の読み取り（AI要約取得時）

### 2.4 トークン管理
- **アクセストークンの有効期限**: 1時間
- **リフレッシュトークンの有効期限**: 90日間
- **推奨事項**: リフレッシュトークンは常に最新のものを使用

## 3. 録画取得API

### 3.1 ユーザー録画リスト取得
```
GET https://api.zoom.us/v2/users/{userId}/recordings
```

#### パラメータ
| パラメータ | 型 | 必須 | 説明 |
|------------|----|----- |------|
| userId | string | ✓ | ユーザーID（"me"で現在のユーザー） |
| from | string | | 開始日（YYYY-MM-DD形式） |
| to | string | | 終了日（YYYY-MM-DD形式） |
| page_size | integer | | 1ページあたりの結果数（デフォルト：30、最大：300） |
| page_number | integer | | ページ番号（1から開始） |
| mc | boolean | | 会議コンテンツを含むかどうか |
| trash | boolean | | 削除済み録画を含むかどうか |

#### レスポンス例
```json
{
  "from": "2025-01-01",
  "to": "2025-01-31",
  "page_count": 1,
  "page_size": 30,
  "total_records": 5,
  "meetings": [
    {
      "uuid": "4444AAAiAAAAAiAiAiiAii==",
      "id": 123456789,
      "account_id": "BdLyCvzyTkuVOqiuR_ZgVg",
      "host_id": "uLoTyVzVTrq4lEKiUJCw",
      "topic": "会議の件名",
      "type": 8,
      "start_time": "2025-01-15T10:00:00Z",
      "timezone": "Asia/Tokyo",
      "duration": 60,
      "total_size": 285212672,
      "recording_count": 4,
      "recording_files": [
        {
          "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
          "meeting_id": "4444AAAiAAAAAiAiAiiAii==",
          "recording_start": "2025-01-15T10:00:00Z",
          "recording_end": "2025-01-15T11:00:00Z",
          "file_type": "MP4",
          "file_extension": "MP4",
          "file_size": 268435456,
          "play_url": "https://zoom.us/rec/play/abc123...",
          "download_url": "https://zoom.us/rec/download/abc123...",
          "status": "completed",
          "recording_type": "shared_screen_with_speaker_view"
        },
        {
          "id": "b2c3d4e5-f6g7-8901-bcde-f23456789012",
          "meeting_id": "4444AAAiAAAAAiAiAiiAii==",
          "recording_start": "2025-01-15T10:00:00Z",
          "recording_end": "2025-01-15T11:00:00Z",
          "file_type": "M4A",
          "file_extension": "M4A",
          "file_size": 16777216,
          "play_url": "https://zoom.us/rec/play/def456...",
          "download_url": "https://zoom.us/rec/download/def456...",
          "status": "completed",
          "recording_type": "audio_only"
        },
        {
          "id": "c3d4e5f6-g7h8-9012-cdef-345678901234",
          "meeting_id": "4444AAAiAAAAAiAiAiiAii==",
          "file_type": "CHAT",
          "file_extension": "TXT",
          "file_size": 1024,
          "download_url": "https://zoom.us/rec/download/ghi789...",
          "status": "completed",
          "recording_type": "chat_file"
        },
        {
          "id": "d4e5f6g7-h8i9-0123-defg-456789012345",
          "meeting_id": "4444AAAiAAAAAiAiAiiAii==",
          "file_type": "TRANSCRIPT",
          "file_extension": "VTT",
          "file_size": 2048,
          "download_url": "https://zoom.us/rec/download/jkl012...",
          "status": "completed",
          "recording_type": "audio_transcript"
        }
      ]
    }
  ]
}
```

### 3.2 録画ファイルの種類
| file_type | recording_type | 説明 |
|-----------|----------------|------|
| MP4 | shared_screen_with_speaker_view | 画面共有+スピーカービュー |
| MP4 | gallery_view | ギャラリービュー |
| MP4 | shared_screen | 画面共有のみ |
| M4A | audio_only | 音声のみ |
| CHAT | chat_file | チャットログ |
| TRANSCRIPT | audio_transcript | 音声転写（VTT形式） |

### 3.3 重要な制約
- **取得可能期間**: 過去30日間のみ（デフォルト）
- **ページネーション**: 必須（大量データの場合）
- **ファイルサイズ**: 各ファイルのサイズは`file_size`フィールドで確認
- **パスワード保護**: パスワード保護された録画は特別な処理が必要

## 4. AI要約取得API

### 4.1 現在の状況（2025年1月）
Zoom AI Companionの要約機能は以下の状況です：

#### 利用可能な機能
- **会議要約の自動生成**: 会議終了10-15分後にメール通知
- **要約の保存**: Zoom Webクライアントに保存
- **手動アクセス**: Zoom Webインターフェースから閲覧可能

#### API提供状況
- **公式API**: 明確なドキュメントは確認できず
- **要望多数**: 開発者フォーラムでAPI提供の要望が多数
- **可能性のあるエンドポイント**: `GET /meetings/{meetingId}/summary`（未確認）

#### 実装時の注意点
- API仕様が不明確なため、Web UI経由での取得を検討
- 将来的にAPI提供される可能性が高い
- 現時点では録画データと併せてWeb UIからの取得を推奨

## 5. レート制限

### 5.1 アカウント種別による制限
#### Free アカウント
- **Light APIs**: 4リクエスト/秒、6,000リクエスト/日
- **Medium APIs**: より制限的
- **Heavy APIs**: より制限的
- **Resource-intensive APIs**: 最も制限的

#### Pro アカウント
- **Light APIs**: 30リクエスト/秒
- **Medium APIs**: 制限あり
- **Heavy APIs**: 制限あり
- **Resource-intensive APIs**: 制限あり

#### Business+ アカウント（推奨）
- **Light APIs**: 80リクエスト/秒
- **Medium APIs**: 60リクエスト/秒
- **Heavy APIs**: 40リクエスト/秒 + 60,000リクエスト/日
- **Resource-intensive APIs**: 20リクエスト/秒 + 60,000リクエスト/日

### 5.2 特殊な制限
- **会議作成API**: 100リクエスト/日（ユーザー単位）
- **録画ダウンロード**: 通常Heavy APIsに分類
- **認証関連**: 通常Light APIsに分類

### 5.3 エラーレスポンス
#### 429 Too Many Requests
```json
{
  "code": 429,
  "message": "You have reached the maximum per-second rate limit for this API"
}
```

または

```json
{
  "code": 429,
  "message": "You have reached the maximum daily rate limit for this API"
}
```

### 5.4 レート制限回避策
1. **指数バックオフ**: エラー時に待機時間を段階的に延長
2. **リクエスト間隔調整**: APIカテゴリに応じた適切な間隔
3. **Webhook活用**: ポーリングの代わりにWebhookを使用
4. **レスポンスキャッシュ**: 同一データの重複取得を避ける
5. **並行処理制限**: 同時実行数を制限

## 6. エラーハンドリング

### 6.1 一般的なHTTPステータスコード
| コード | 説明 | 対処法 |
|--------|------|--------|
| 200 | 成功 | 正常処理 |
| 400 | Bad Request | リクエストパラメータを確認 |
| 401 | Unauthorized | アクセストークンを更新 |
| 403 | Forbidden | スコープまたは権限を確認 |
| 404 | Not Found | リソースの存在を確認 |
| 429 | Too Many Requests | レート制限対応 |
| 500 | Internal Server Error | リトライ処理 |

### 6.2 認証エラー
```json
{
  "code": 124,
  "message": "Invalid access token."
}
```

### 6.3 スコープエラー
```json
{
  "code": 200,
  "message": "Invalid access token, does not contain scopes: [recording:read]"
}
```

## 7. ベストプラクティス

### 7.1 認証管理
- アクセストークンの自動更新機能を実装
- トークンの安全な保存（暗号化推奨）
- リフレッシュトークンの適切な管理

### 7.2 APIアクセス最適化
- 必要最小限のデータのみ取得
- ページネーションの適切な活用
- 並行ダウンロードの制限（5-10ファイル同時程度）

### 7.3 エラー処理
- 全APIエラーに対する適切なハンドリング
- ユーザーフレンドリーなエラーメッセージ
- 自動リトライ機能（指数バックオフ）

### 7.4 セキュリティ
- HTTPS通信の強制
- Client SecretとTokenの安全な管理
- ログにおける機密情報の除外

## 8. 実装参考情報

### 8.1 HTTPヘッダー
```
Authorization: Bearer {access_token}
Content-Type: application/json
User-Agent: ZoomVideoMover/1.0
```

### 8.2 日付フォーマット
- **ISO 8601形式**: `2025-01-15T10:00:00Z`
- **日付のみ**: `2025-01-15`（パラメータ用）

### 8.3 ファイル名の注意点
- 日本語文字を含む可能性
- 特殊文字のサニタイゼーションが必要
- Windowsパスの制限を考慮

## 9. 制限事項・注意点

### 9.1 録画データの制限
- **保存期間**: Zoomアカウント設定による（通常30日〜無制限）
- **パスワード保護**: 現在のOAuth APIでは取得不可の場合あり
- **削除済みファイル**: `trash=true`パラメータで取得可能

### 9.2 API仕様の変更
- Zoom APIは定期的に更新される
- 非推奨APIの段階的廃止
- 新機能の追加（AI要約API等）

### 9.3 アカウント依存の機能
- AI Companion機能はアカウント設定による
- 録画機能の有効化が必要
- 管理者権限による制限の可能性

---

## 付録: 参考リンク

- [Zoom Developer Documentation](https://developers.zoom.us/docs/api/)
- [OAuth Authentication](https://developers.zoom.us/docs/integrations/oauth/)
- [Rate Limits](https://developers.zoom.us/docs/api/rate-limits/)
- [Zoom Developer Forum](https://devforum.zoom.us/)

---

**最終更新**: 2025年1月31日  
**対象バージョン**: Zoom API v2  
**プロジェクト**: Zoom Video Mover