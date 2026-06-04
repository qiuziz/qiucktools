#!/bin/bash
# 清理鸿蒙应用数据（通过 hdc）
# 用法: ./clean-hmos-app.sh <bundle-name>
# 示例: ./clean-hmos-app.sh com.anjuke.home

BUNDLE_NAME="$1"

if [ -z "$BUNDLE_NAME" ]; then
    echo "错误: 请指定应用包名"
    echo "用法: $0 <bundle-name>"
    echo "示例: $0 com.anjuke.home"
    exit 1
fi

echo "正在清理 $BUNDLE_NAME 的数据..."

RESULT=$(hdc shell "bm clean -n $BUNDLE_NAME -d" 2>&1)

if echo "$RESULT" | grep -q "successfully"; then
    echo "✓ $BUNDLE_NAME 数据清理成功"
    exit 0
else
    echo "✗ 清理失败"
    echo "$RESULT"
    exit 1
fi
