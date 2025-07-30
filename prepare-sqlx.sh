#!/bin/bash

echo "SQLx準備ファイルを生成します..."

# 一時的なSQLiteデータベースを作成
export DATABASE_URL="sqlite:///tmp/prepare_db.sqlite"
rm -f /tmp/prepare_db.sqlite
touch /tmp/prepare_db.sqlite

# マイグレーションを実行
echo "マイグレーションを実行中..."
sqlx migrate run

# 準備ファイルを生成
echo "SQLx準備ファイルを生成中..."
cargo sqlx prepare

echo "完了！.sqlx/ディレクトリに準備ファイルが生成されました。"