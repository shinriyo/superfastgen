#!/bin/bash

echo "🔄 Updating tree-sitter-dart..."

# tree-sitter-dartを最新版に更新
git submodule update --remote tree-sitter-dart

# 更新されたかチェック
if [ $? -eq 0 ]; then
    echo "✅ tree-sitter-dart updated successfully"
    
    # ビルドしてテスト
    echo "🔨 Rebuilding project..."
    cargo build
    
    if [ $? -eq 0 ]; then
        echo "✅ Build successful after update"
        echo "🧪 Running tests..."
        cargo test
    else
        echo "❌ Build failed after update"
        exit 1
    fi
else
    echo "❌ Failed to update tree-sitter-dart"
    exit 1
fi

echo "🎉 Update completed successfully!" 