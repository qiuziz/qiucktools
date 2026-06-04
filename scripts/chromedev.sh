#!/bin/bash
set -euo pipefail

if ! command -v hdc >/dev/null 2>&1 && [[ -n "${HOME:-}" ]]; then
    export PATH="$HOME/bin:$PATH"
fi
export PATH="$PATH:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"

PORT=9222

usage() {
    cat <<EOF
用法: $0 [anjuke|wuba]

说明:
  清理现有 hdc fport 映射，并将 Harmony WebView DevTools 转发到 tcp:9222。
  当 hdc 检测到多台真机时，会弹出 macOS 原生选择框让用户挑选。

参数:
  anjuke  com.anjuke.home（默认）
  wuba    com.wuba.life

环境变量:
  HDC_DEVICE  跳过设备选择，直接使用指定的设备 serial
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

DEVICE_FLAG=()

list_targets() {
    hdc list targets
}

# Output: serial \t address \t state
pick_device() {
    if [[ -n "${HDC_DEVICE:-}" ]]; then
        echo "$HDC_DEVICE"
        return 0
    fi

    local targets
    targets=$(list_targets || true)
    local -a serials=()
    local -a labels=()
    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        local serial addr state
        read -r serial addr state _ <<< "$line"
        [[ -z "$serial" ]] && continue
        serials+=("$serial")
        labels+=("$serial  $addr  [$state]")
    done <<< "$targets"

    local count="${#serials[@]}"
    if [[ "$count" -eq 0 ]]; then
        echo "错误: 未检测到任何 hdc 设备，请先连接真机后重试。" >&2
        return 2
    fi
    if [[ "$count" -eq 1 ]]; then
        echo "检测到 1 台设备: ${labels[0]}" >&2
        echo "${serials[0]}"
        return 0
    fi

    echo "检测到 $count 台设备，请选择:" >&2
    local -a osa_choices=()
    local label
    for label in "${labels[@]}"; do
        osa_choices+=("$label")
    done

    local script
    # AppleScript is happiest when `choose from list` receives the list literal
    # directly, rather than going through a `set` variable. Round-tripping
    # through a variable surfaced syntax errors in Tauri child processes
    # (where `osascript` runs against the GUI AppleScript compiler, not the
    # headless parser). Build the list literal as a single line.
    local list_literal
    list_literal=$(printf '"%s", ' "${osa_choices[@]}")
    list_literal="${list_literal%, }"  # strip trailing ", "
    script="return choose from list { ${list_literal} } with title \"AJK WebView DevTools\" with prompt \"检测到 $count 台 hdc 设备,请选择要调试的真机:\" OK button name \"选择\" cancel button name \"取消\""

    local chosen
    if ! chosen=$(osascript -e "$script" 2>&1); then
        echo "错误: 设备选择失败: $chosen" >&2
        return 3
    fi

    if [[ -z "$chosen" || "$chosen" == "false" ]]; then
        echo "已取消设备选择。" >&2
        return 1
    fi

    local picked_serial="${chosen%%  *}"
    if [[ -z "$picked_serial" ]]; then
        echo "错误: 未能解析选中的设备 serial: $chosen" >&2
        return 3
    fi
    echo "已选择设备: $picked_serial" >&2
    echo "$picked_serial"
}

# pick_device may return non-boolean exit codes (1=cancel, 2=no devices, 3=osascript
# error). Capture before the `if` to avoid `!` flipping the value.
set +e
PICKED=$(pick_device)
PICK_RC=$?
set -e
if [[ "$PICK_RC" -ne 0 ]]; then
    exit "$PICK_RC"
fi
DEVICE_FLAG=("-t" "$PICKED")

remove_existing_forward_port() {
    local port="$1"

    while IFS= read -r line; do
        if [[ -z "$line" || "$line" == *"[Empty]"* || "$line" != *"[Forward]"* ]]; then
            continue
        fi

        local first second third
        read -r first second third _ <<< "$line"

        local local_node remote_node
        if [[ "$first" == tcp:* ]]; then
            local_node="$first"
            remote_node="$second"
        else
            local_node="$second"
            remote_node="$third"
        fi

        if [[ "$local_node" != "tcp:$port" || -z "$remote_node" ]]; then
            continue
        fi

        echo "删除映射: $local_node -> $remote_node"
        hdc "${DEVICE_FLAG[@]}" fport rm "$local_node" "$remote_node" || true
    done < <(hdc "${DEVICE_FLAG[@]}" fport ls)
}

echo "目标应用: $APP"
if [[ "${#DEVICE_FLAG[@]}" -gt 0 ]]; then
    echo "目标设备: ${DEVICE_FLAG[1]}"
fi

echo "清理已有 tcp:$PORT hdc fport 映射..."
remove_existing_forward_port "$PORT"

SOCKET_NAME=$(hdc "${DEVICE_FLAG[@]}" shell "ps -ef | grep $APP | grep -v grep" || true)
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
echo "添加映射: tcp:$PORT -> localabstract:webview_devtools_remote_$PID"
FPORT_OUTPUT=$(hdc "${DEVICE_FLAG[@]}" fport "tcp:$PORT" localabstract:webview_devtools_remote_"$PID" 2>&1)
echo "$FPORT_OUTPUT"
if [[ "$FPORT_OUTPUT" == *"[Fail]"* ]]; then
    echo "错误: hdc fport 添加映射失败。"
    exit 1
fi

echo ""
echo "当前 hdc fport 映射:"
FPORT_LIST=$(hdc "${DEVICE_FLAG[@]}" fport ls)
echo "$FPORT_LIST"
if ! grep -Fq "tcp:$PORT localabstract:webview_devtools_remote_$PID" <<< "$FPORT_LIST"; then
    echo "错误: 未确认到 tcp:$PORT -> webview_devtools_remote_$PID 映射。"
    exit 1
fi

echo ""
echo "DevTools 地址: http://127.0.0.1:$PORT"
