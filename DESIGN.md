# HyperDashi バックエンド詳細設計書

## 1. システム概要

HyperDashiは、情報メディアシステム局の物品管理システム「dashi」の発展版・汎用版である。グラフデータベースと検索エンジンを使用していた従来システムから、運用の簡素化を目的としてRDBベースのシステムに移行する。

### 1.1 技術スタック
- **言語**: Rust
- **Webフレームワーク**: Axum
- **データベース**: PostgreSQL (本番環境) / SQLite (開発環境)
- **ORM**: SQLx
- **ストレージ**: S3互換オブジェクトストレージ (本番環境) / ローカルファイルシステム (開発環境)
- **API形式**: REST API

## 2. データベース設計

### 2.1 物品情報テーブル (items)

```sql
CREATE TABLE items (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    label_id VARCHAR(50) UNIQUE NOT NULL,
    model_number VARCHAR(255),
    remarks TEXT,
    purchase_year INTEGER,
    purchase_amount DECIMAL(12, 2),
    useful_life INTEGER,
    is_depreciable BOOLEAN DEFAULT FALSE,
    connection_names TEXT[], -- PostgreSQL配列型、SQLiteではJSON
    cable_color_pattern TEXT[], -- PostgreSQL配列型、SQLiteではJSON
    storage_locations TEXT[], -- PostgreSQL配列型、SQLiteではJSON
    is_on_loan BOOLEAN DEFAULT FALSE,
    label_type VARCHAR(20) CHECK (label_type IN ('qr', 'barcode', 'none')),
    is_disposed BOOLEAN DEFAULT FALSE,
    image_url TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- インデックス
CREATE INDEX idx_items_label_id ON items(label_id);
CREATE INDEX idx_items_name ON items(name);
CREATE INDEX idx_items_is_on_loan ON items(is_on_loan);
CREATE INDEX idx_items_is_disposed ON items(is_disposed);
```

### 2.2 貸出管理テーブル (loans)

```sql
CREATE TABLE loans (
    id SERIAL PRIMARY KEY,
    item_id INTEGER NOT NULL REFERENCES items(id),
    student_number VARCHAR(20) NOT NULL,
    student_name VARCHAR(100) NOT NULL,
    organization VARCHAR(255),
    loan_date TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    return_date TIMESTAMP WITH TIME ZONE,
    remarks TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- インデックス
CREATE INDEX idx_loans_item_id ON loans(item_id);
CREATE INDEX idx_loans_student_number ON loans(student_number);
CREATE INDEX idx_loans_return_date ON loans(return_date);
```

### 2.3 画像管理テーブル (images) - オプション

```sql
CREATE TABLE images (
    id SERIAL PRIMARY KEY,
    file_name VARCHAR(255) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    storage_type VARCHAR(20) CHECK (storage_type IN ('s3', 'local')),
    storage_path TEXT NOT NULL,
    size_bytes BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

## 3. API設計

### 3.1 物品管理API

#### 3.1.1 物品一覧取得
- **エンドポイント**: `GET /api/v1/items`
- **クエリパラメータ**:
  - `page` (optional): ページ番号
  - `per_page` (optional): 1ページあたりの件数
  - `search` (optional): 検索キーワード
  - `is_on_loan` (optional): 貸出中フィルタ
  - `is_disposed` (optional): 廃棄済みフィルタ
- **レスポンス**:
```json
{
  "items": [
    {
      "id": 1,
      "name": "HDMIケーブル 3m",
      "label_id": "HYP-A001",
      "model_number": "HDMI-3M-V2",
      "remarks": "端子部分に少し傷あり",
      "purchase_year": 2023,
      "purchase_amount": 1500.00,
      "useful_life": 5,
      "is_depreciable": false,
      "connection_names": ["HDMI Type-A", "HDMI Type-A"],
      "cable_color_pattern": ["赤", "青", "赤"],
      "storage_locations": ["部屋A", "ラックX", "コンテナα"],
      "is_on_loan": false,
      "label_type": "qr",
      "is_disposed": false,
      "image_url": "https://storage.example.com/images/hdmi-cable.jpg",
      "created_at": "2023-04-01T10:00:00Z",
      "updated_at": "2023-04-01T10:00:00Z"
    }
  ],
  "total": 150,
  "page": 1,
  "per_page": 20
}
```

#### 3.1.2 物品詳細取得
- **エンドポイント**: `GET /api/v1/items/{id}`
- **レスポンス**: 単一の物品オブジェクト

#### 3.1.3 物品登録
- **エンドポイント**: `POST /api/v1/items`
- **リクエストボディ**:
```json
{
  "name": "HDMIケーブル 3m",
  "label_id": "HYP-A001",
  "model_number": "HDMI-3M-V2",
  "remarks": "端子部分に少し傷あり",
  "purchase_year": 2023,
  "purchase_amount": 1500.00,
  "useful_life": 5,
  "is_depreciable": false,
  "connection_names": ["HDMI Type-A", "HDMI Type-A"],
  "cable_color_pattern": ["赤", "青", "赤"],
  "storage_locations": ["部屋A", "ラックX", "コンテナα"],
  "label_type": "qr"
}
```

#### 3.1.4 物品更新
- **エンドポイント**: `PUT /api/v1/items/{id}`
- **リクエストボディ**: 物品登録と同じ（全項目更新）

#### 3.1.5 物品部分更新
- **エンドポイント**: `PATCH /api/v1/items/{id}`
- **リクエストボディ**: 更新したいフィールドのみ

#### 3.1.6 物品削除
- **エンドポイント**: `DELETE /api/v1/items/{id}`
- **説明**: 論理削除ではなく物理削除（ミス入力の修正用）

#### 3.1.7 物品廃棄/譲渡
- **エンドポイント**: `POST /api/v1/items/{id}/dispose`
- **説明**: is_disposedフラグを立てる

#### 3.1.8 ラベルIDによる物品検索
- **エンドポイント**: `GET /api/v1/items/by-label/{label_id}`
- **説明**: QRコード/バーコード読み取り時の検索用

### 3.2 貸出管理API

#### 3.2.1 貸出登録
- **エンドポイント**: `POST /api/v1/loans`
- **リクエストボディ**:
```json
{
  "item_id": 1,
  "student_number": "21001234",
  "student_name": "山田太郎",
  "organization": "第74回総合祭実行委員会",
  "remarks": "イベント用機材"
}
```

#### 3.2.2 返却処理
- **エンドポイント**: `POST /api/v1/loans/{id}/return`
- **リクエストボディ**:
```json
{
  "return_date": "2023-04-10T15:00:00Z",
  "remarks": "問題なく返却"
}
```

#### 3.2.3 貸出履歴取得
- **エンドポイント**: `GET /api/v1/loans`
- **クエリパラメータ**:
  - `item_id` (optional): 物品ID
  - `student_number` (optional): 学籍番号
  - `active_only` (optional): 未返却のみ

#### 3.2.4 貸出詳細取得
- **エンドポイント**: `GET /api/v1/loans/{id}`

### 3.3 画像アップロードAPI

#### 3.3.1 画像アップロード
- **エンドポイント**: `POST /api/v1/images/upload`
- **Content-Type**: `multipart/form-data`
- **レスポンス**:
```json
{
  "url": "https://storage.example.com/images/abc123.jpg"
}
```

### 3.4 一括操作API

#### 3.4.1 物品一括登録
- **エンドポイント**: `POST /api/v1/items/bulk`
- **リクエストボディ**: 物品オブジェクトの配列

#### 3.4.2 物品一括更新
- **エンドポイント**: `PUT /api/v1/items/bulk`
- **リクエストボディ**: 更新対象の物品オブジェクトの配列

## 4. アプリケーション構造

### 4.1 ディレクトリ構造
```
hyperdashi-server/
├── src/
│   ├── main.rs              # エントリーポイント
│   ├── config.rs            # 設定管理
│   ├── db/
│   │   ├── mod.rs          # データベース接続管理
│   │   └── migrations/     # SQLマイグレーション
│   ├── models/
│   │   ├── mod.rs
│   │   ├── item.rs         # 物品モデル
│   │   └── loan.rs         # 貸出モデル
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── items.rs        # 物品ハンドラー
│   │   ├── loans.rs        # 貸出ハンドラー
│   │   └── images.rs       # 画像ハンドラー
│   ├── services/
│   │   ├── mod.rs
│   │   ├── item_service.rs # 物品ビジネスロジック
│   │   ├── loan_service.rs # 貸出ビジネスロジック
│   │   └── storage.rs      # ストレージ抽象化
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── validation.rs   # バリデーション
│   │   └── label.rs        # ラベルID生成
│   └── error.rs            # エラー型定義
├── Cargo.toml
├── .env.example
└── README.md
```

### 4.2 主要コンポーネント

#### 4.2.1 設定管理 (config.rs)
```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub storage_type: StorageType,
    pub s3_config: Option<S3Config>,
    pub local_storage_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub enum StorageType {
    S3,
    Local,
}
```

#### 4.2.2 ストレージ抽象化 (storage.rs)
```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn upload(&self, data: Vec<u8>, filename: &str) -> Result<String>;
    async fn delete(&self, url: &str) -> Result<()>;
}

pub struct S3Storage { /* ... */ }
pub struct LocalStorage { /* ... */ }
```

## 5. セキュリティ考慮事項

### 5.1 認証・認可
- 初期版では認証なし（内部システムのため）
- 将来的にはJWT等による認証を実装予定

### 5.2 入力検証
- ラベルIDの形式検証（英数字、I/O除外）
- SQLインジェクション対策（SQLx使用）
- ファイルアップロードのサイズ制限とMIMEタイプ検証

### 5.3 データ保護
- 個人情報（学籍番号、氏名）の適切な管理
- HTTPSによる通信の暗号化

## 6. パフォーマンス最適化

### 6.1 データベース
- 適切なインデックスの設定
- N+1問題の回避
- コネクションプーリング

### 6.2 画像処理
- 画像のリサイズとサムネイル生成
- CDN利用による配信最適化（将来）

### 6.3 キャッシング
- 頻繁にアクセスされる物品情報のキャッシング（将来）

## 7. 運用・保守

### 7.1 ロギング
- 構造化ログの出力
- エラートラッキング

### 7.2 モニタリング
- ヘルスチェックエンドポイント
- メトリクス収集（将来）

### 7.3 バックアップ
- データベースの定期バックアップ
- 画像データのバックアップ

## 8. 今後の拡張予定

- 認証・認可機能
- 物品の予約機能
- 統計・レポート機能
- モバイルアプリ対応API
- WebSocket による リアルタイム更新