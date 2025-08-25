#!/bin/bash

# GLIBCバージョン問題を解決するためのテストビルドスクリプト

echo "=== Building hyperdashi-server with bookworm base ==="
echo ""

# ビルド（キャッシュなし）
echo "Building Docker image without cache..."
docker build --no-cache -t hyperdashi-server-test . 2>&1 | tee build.log

# ビルドが成功したか確認
if [ $? -eq 0 ]; then
    echo ""
    echo "=== Build successful! ==="
    echo ""
    
    # GLIBCバージョンを確認
    echo "Checking GLIBC version in the container..."
    docker run --rm hyperdashi-server-test ldd --version | head -n1
    
    echo ""
    echo "Checking binary dependencies..."
    docker run --rm hyperdashi-server-test ldd /usr/local/bin/hyperdashi-server | grep -E "(GLIBC|libc\.so)"
    
    echo ""
    echo "Testing the application startup..."
    docker run --rm -e DATABASE_URL="sqlite://:memory:" hyperdashi-server-test hyperdashi-server --version 2>&1 || echo "Version check failed (expected if --version is not implemented)"
else
    echo ""
    echo "=== Build failed! ==="
    echo "Check build.log for details"
    tail -50 build.log
    exit 1
fi