# ビルドステージ
FROM rust:latest AS builder

# 作業ディレクトリを設定
WORKDIR /usr/src/hyperdashi

# Rustツールチェインを更新
RUN rustup update stable

# ソースコードをコピー
COPY . .

# アプリケーションをビルド（オフラインモード）
ENV SQLX_OFFLINE=true
# リンカーをmoldに設定してビルド時間とメモリ使用量を削減
RUN apt-get update && apt-get install -y mold && rm -rf /var/lib/apt/lists/*
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# メモリ使用量とビルドの詳細を監視
RUN echo "Available memory:" && free -h && \
    echo "Available disk space:" && df -h && \
    echo "Starting cargo build..." && \
    (cargo build --release --verbose 2>&1 | tee /tmp/build.log; exit ${PIPESTATUS[0]}) && \
    echo "Build completed successfully. Checking result:" && \
    ls -la target/release/ && \
    echo "Binary details:" && \
    file target/release/hyperdashi-server || echo "file command not available" || \
    (echo "Build failed! Last 50 lines of output:" && tail -50 /tmp/build.log && exit 1)

# ビルドされたバイナリのサイズを検証
RUN ls -lh target/release/hyperdashi-server && \
    test -s target/release/hyperdashi-server || (echo "Binary is empty!" && exit 1) && \
    [ $(stat -f%z target/release/hyperdashi-server 2>/dev/null || stat -c%s target/release/hyperdashi-server) -gt 1000000 ] || (echo "Binary too small: $(stat -f%z target/release/hyperdashi-server 2>/dev/null || stat -c%s target/release/hyperdashi-server) bytes" && exit 1)

# 実行ステージ
FROM debian:bookworm-slim

# 必要なランタイムライブラリをインストール
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# 非rootユーザーを作成
RUN useradd -m -u 1000 -U hyperdashi

# アプリケーションをコピー
COPY --from=builder /usr/src/hyperdashi/target/release/hyperdashi-server /usr/local/bin/hyperdashi-server

# マイグレーションファイルをコピー
COPY --from=builder /usr/src/hyperdashi/migrations /app/migrations

# 初期化スクリプトをコピー
COPY init.sh /usr/local/bin/init.sh
RUN sed -i 's/\r$//' /usr/local/bin/init.sh && \
    chmod +x /usr/local/bin/init.sh

# 作業ディレクトリを設定
WORKDIR /app

# データディレクトリとアップロードディレクトリを作成
RUN mkdir -p /app/data /app/uploads

# 所有権を変更
RUN chown -R hyperdashi:hyperdashi /app

# hyperdashiユーザーに切り替え
USER hyperdashi

# ポートを公開
EXPOSE 8080

# ヘルスチェック
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

# データディレクトリとアップロードディレクトリを作成
RUN mkdir -p /app/data /app/uploads && chown -R hyperdashi:hyperdashi /app/data /app/uploads

# アプリケーションを直接実行
CMD ["hyperdashi-server"]