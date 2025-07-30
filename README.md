# HyperDashi Server

物品管理システムのRustバックエンド。

## 開発

```bash
cargo run
```

## 技術スタック

- Rust + Axum
- SQLx (PostgreSQL/SQLite)
- AWS SDK (S3互換ストレージ)

## 環境変数

主要な設定：

```env
DATABASE_URL=postgresql://user:password@localhost/hyperdashi
STORAGE_TYPE=s3
S3_ENDPOINT=http://localhost:9000
S3_BUCKET_NAME=hyperdashi-images
```