#!/bin/bash
set -euo pipefail

usage() {
    cat <<EOF
用法: $0 [anjuke|wuba]

说明:
  清理现有 hdc fport 映射，并将 Harmony WebView DevTools 转发到 tcp:9222。

参数:
  anjuke  com.anjuke.home（默认）
  wuba    com.wuba.life
EOF
}

APPNAME="${1:-anjuke}"

case "$APPNAME" in
    anjuke)
        APP="com.anjuke.home"
        ;;
    wuba)
        APP="com.wuba.life"
        ;;
    -h|--help)
        usage
        exit 0
        ;;
    *)
        echo "错误: 不支持的应用: $APPNAME"
        usage
        exit 1
        ;;
esac

if ! command -v hdc >/dev/null 2>&1; then
    echo "错误: 未找到 hdc，请先安装并配置 HarmonyOS hdc 命令。"
    exit 1
fi

echo "清理已有 hdc fport 映射..."
while IFS= read -r line; do
    if [[ -n "$line" && "$line" != *"[Empty]"* ]]; then
        hdc fport rm "$line" || true
    fi
done < <(hdc fport ls)

echo "目标应用: $APP"

SOCKET_NAME=$(hdc shell "ps -ef | grep $APP | grep -v grep" || true)
if [ -z "$SOCKET_NAME" ]; then
    echo "错误: 未找到应用进程，请确认应用已在设备上启动并开启 WebView 调试。"
    exit 1
fi

PID=$(echo "$SOCKET_NAME" | awk 'NR==1 {print $2}')
if [ -z "$PID" ]; then
    echo "错误: 无法提取应用进程 ID。"
    exit 1
fi

echo "应用进程 PID: $PID"
echo "添加映射: tcp:9222 -> localabstract:webview_devtools_remote_$PID"
hdc fport tcp:9222 localabstract:webview_devtools_remote_"$PID"

echo ""
echo "当前 hdc fport 映射:"
hdc fport ls

echo ""
echo "DevTools 地址: http://127.0.0.1:9222"
