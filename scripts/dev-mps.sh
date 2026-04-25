#!/bin/bash
set -e

MINIAPP_HOST_DIR="${MINIAPP_HOST_DIR:-/Users/qiuz/work/brain/projects/miniapp-host/code}"
CONFIG_FILE="$MINIAPP_HOST_DIR/mps-config/dev.mps.json"

usage() {
    cat <<EOF
用法: $0 -p <package> -v <version> [-p <package> -v <version> ...] [-c <config.json>]

示例:
  $0 -p host-im -v 4819USER24104
  $0 -p host-im -v 4819USER24104 -p host-search -v 0.1.9
  $0 -p @mps/weapp-xf -v 0.1.0 -c extra.json

参数:
  -p  包名（key）
  -v  版本值（自动判断: 含USER或长度>15为versionName，否则为version）
  -c  高级参数JSON文件（可选）

环境变量:
  MINIAPP_HOST_DIR  miniapp-host 仓库路径，默认 /Users/qiuz/work/brain/projects/miniapp-host/code
EOF
    exit 1
}

PACKAGES=()
EXTRA_CONFIG=""
while getopts "p:v:c:h" opt; do
    case $opt in
        p) PACKAGES+=("$OPTARG");;
        v) PACKAGES+=("$OPTARG");;
        c) EXTRA_CONFIG="$OPTARG";;
        h) usage;;
        *) usage;;
    esac
done

if [ ${#PACKAGES[@]} -lt 2 ] || [ $((${#PACKAGES[@]} % 2)) -ne 0 ]; then
    echo "错误: 需要成对的 -p 和 -v 参数"
    usage
fi

if [ ! -d "$MINIAPP_HOST_DIR" ]; then
    echo "错误: miniapp-host 仓库不存在: $MINIAPP_HOST_DIR"
    exit 1
fi

determine_version_type() {
    local ver="$1"
    if [[ "$ver" == *"USER"* ]] || [ ${#ver} -gt 15 ]; then
        echo "versionName"
    elif [[ "$ver" =~ ^[0-9]+\.[0-9]+(\.[0-9]+)?$ ]]; then
        echo "version"
    else
        echo "versionName"
    fi
}

load_extra_config() {
    local config_file="$1"
    if [ -n "$config_file" ] && [ -f "$config_file" ]; then
        cat "$config_file"
    else
        echo "{}"
    fi
}

if ! command -v jq &> /dev/null; then
    echo "错误: 需要安装 jq"
    echo "  macOS: brew install jq"
    exit 1
fi

mkdir -p "$(dirname "$CONFIG_FILE")"
if [ ! -f "$CONFIG_FILE" ]; then
    echo "{}" > "$CONFIG_FILE"
fi

EXISTING=$(cat "$CONFIG_FILE")
EXTRA_JSON=$(load_extra_config "$EXTRA_CONFIG")

for ((i=0; i<${#PACKAGES[@]}; i+=2)); do
    PKG="${PACKAGES[i]}"
    VER="${PACKAGES[i+1]}"
    VER_TYPE=$(determine_version_type "$VER")

    echo "处理: $PKG -> $VER (类型: $VER_TYPE)"

    if [ "$VER_TYPE" = "versionName" ]; then
        if [ "$EXTRA_JSON" = "{}" ]; then
            NEW_PKG="{\"versionName\": \"$VER\"}"
        else
            NEW_PKG=$(echo "$EXTRA_JSON" | jq --arg v "$VER" '. + {versionName: $v}')
        fi
    else
        if [ "$EXTRA_JSON" = "{}" ]; then
            NEW_PKG="{\"version\": \"$VER\"}"
        else
            NEW_PKG=$(echo "$EXTRA_JSON" | jq --arg v "$VER" '. + {version: $v}')
        fi
    fi

    if echo "$EXISTING" | jq -e ".$PKG" > /dev/null 2>&1; then
        EXISTING=$(echo "$EXISTING" | jq --arg pkg "$PKG" --argjson val "$NEW_PKG" '.[$pkg] = $val')
    else
        EXISTING=$(echo "$EXISTING" | jq --arg pkg "$PKG" --argjson val "$NEW_PKG" '. + {($pkg): $val}')
    fi
done

echo "$EXISTING" | jq '.' > "$CONFIG_FILE"

cd "$MINIAPP_HOST_DIR"
git add mps-config/dev.mps.json

if git diff --cached --quiet; then
    echo "配置未变更，无需提交"
else
    MSG="chore: update dev.mps.json"
    git commit -m "$MSG"
    git push
    echo "已提交并推送"
fi

echo ""
echo "最终配置:"
cat "$CONFIG_FILE"
