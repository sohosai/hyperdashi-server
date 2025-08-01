# ビルドステージ
FROM rust:latest AS builder

# 作業ディレクトリを設定
WORKDIR /usr/src/hyperdashi

# 依存関係のキャッシュのためにCargo.tomlを先にコピー
COPY Cargo.toml ./

# ダミーのmain.rsを作成して依存関係をビルド
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 実際のソースコードをコピー
COPY . .

# アプリケーションをビルド（オフラインモード）
ENV SQLX_OFFLINE=true
RUN cargo build --release

# 実行ステージ
FROM debian:bookworm-slim

# 必要なランタイムライブラリをインストール
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 非rootユーザーを作成
RUN useradd -m -u 1000 -U hyperdashi

# アプリケーションをコピー
COPY --from=builder /usr/src/hyperdashi/target/release/hyperdashi-server /usr/local/bin/hyperdashi-server

# マイグレーションファイルをコピー
COPY --from=builder /usr/src/hyperdashi/migrations /app/migrations

# 作業ディレクトリを設定
WORKDIR /app

# 所有権を変更
RUN chown -R hyperdashi:hyperdashi /app

# 非rootユーザーに切り替え
USER hyperdashi

# ポートを公開
EXPOSE 8080

# ヘルスチェック
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

# アプリケーションを実行
CMD ["hyperdashi-server"]