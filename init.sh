#!/bin/bash
set -e

# データディレクトリとアップロードディレクトリを作成
mkdir -p /app/data /app/uploads

# 適切な権限を設定
chown -R hyperdashi:hyperdashi /app/data /app/uploads

# hyperdashiユーザーとしてアプリケーションを実行
exec sudo -u hyperdashi hyperdashi-server