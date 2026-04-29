#!/bin/bash
# replace-p7b.sh - 替换 harmonyajkproject 中的签名文件并提交
# 用法: ./replace-p7b.sh <source_p7b> [branch]
#
# 示例:
#   ./replace-p7b.sh ~/Downloads/app/ajk-harmony-debugDebug\(1\).p7b
#   ./replace-p7b.sh ~/Downloads/app/ajk-hap-debug.p7b release-17.36
#
# 参数说明:
#   source_p7b  源 p7b 文件路径（必填）
#   branch      目标分支（默认: 最新 release 分支，如 release-17.36）
#
# 流程:
#   1. 通过 glab 读取远程文件信息
#   2. 对比源文件与远程文件 sha256
#   3. 内容变化时直接调用 GitLab Repository Files API 提交更新

set -e

SOURCE_P7B="${1:-}"
BRANCH="${2:-}"

# ---------- 固定配置 ----------
readonly GITLAB_HOST="igit.58corp.com"
readonly GITLAB_PROJECT="_fe%2Fharmonyajkproject"
readonly REPO_FILE="config/debugSign/ajk-hap-debug.p7b"
readonly COMMIT_MSG="chore: replace ajk-hap-debug.p7b"
# ------------------------------

require_cmd() {
    local cmd="$1"
    if ! command -v "$cmd" > /dev/null 2>&1; then
        echo "错误: 缺少命令 $cmd"
        exit 1
    fi
}

url_encode() {
    jq -rn --arg v "$1" '$v|@uri'
}

# 获取最新 release 分支（按版本号降序排列，取第一个）
get_latest_release_branch() {
    local branches
    require_cmd glab
    require_cmd jq

    branches=$(glab api --hostname "$GITLAB_HOST" "projects/${GITLAB_PROJECT}/repository/branches?search=^release-&per_page=100" --paginate 2>/dev/null \
        | jq -r '.[].name' \
        | grep '^release-' \
        | sort -t'.' -k1,1n -k2,2n -k3,3n \
        | tail -1)

    if [[ -z "$branches" ]]; then
        echo "release-17.36"  # fallback
    else
        echo "$branches"
    fi
}

replace_with_glab() {
    require_cmd glab
    require_cmd jq
    require_cmd base64
    require_cmd shasum

    local file_encoded
    local branch_encoded
    local remote_json
    local remote_sha
    local remote_size
    local new_size
    local new_sha
    local content
    local response
    local commit_id

    file_encoded=$(url_encode "$REPO_FILE")
    branch_encoded=$(url_encode "$BRANCH")

    echo "检查远程文件..."

    if ! remote_json=$(glab api --hostname "$GITLAB_HOST" "projects/${GITLAB_PROJECT}/repository/files/${file_encoded}?ref=${branch_encoded}"); then
        echo "错误: 无法读取远程文件 ${REPO_FILE}（分支: ${BRANCH}）"
        exit 1
    fi

    remote_sha=$(jq -r '.content_sha256 // empty' <<< "$remote_json")
    remote_size=$(jq -r '.size // "unknown"' <<< "$remote_json")
    new_size=$(stat -f%z "$SOURCE_P7B" 2>/dev/null || stat -c%s "$SOURCE_P7B" 2>/dev/null || echo "unknown")
    new_sha=$(shasum -a 256 "$SOURCE_P7B" | awk '{print $1}')

    echo ""
    echo "=== 开始替换签名文件（GitLab CLI） ==="
    echo "源文件:   $SOURCE_P7B"
    echo "目标文件: $REPO_FILE"
    echo "分支:     $BRANCH"
    echo "原文件:   $remote_size bytes"
    echo "新文件:   $new_size bytes"

    if [[ -n "$remote_sha" ]] && [[ "$remote_sha" == "$new_sha" ]]; then
        echo ""
        echo "警告: 文件内容未变化，跳过提交"
        exit 0
    fi

    content=$(base64 < "$SOURCE_P7B" | tr -d '\n')

    echo ""
    echo "提交远程变更..."
    response=$(glab api --hostname "$GITLAB_HOST" --method PUT "projects/${GITLAB_PROJECT}/repository/files/${file_encoded}" \
        -f "branch=${BRANCH}" \
        -f "commit_message=${COMMIT_MSG}" \
        -f "content=${content}" \
        -f "encoding=base64")

    commit_id=$(jq -r '.commit_id // empty' <<< "$response")
    echo ""
    echo "=== 完成 ==="
    echo "分支: $BRANCH"
    if [[ -n "$commit_id" ]]; then
        echo "提交: ${commit_id:0:8}"
    fi
}

# 展开 ~（不用 eval，避免路径含特殊字符时出错）
if [[ "$SOURCE_P7B" == ~/* ]]; then
    SOURCE_P7B="${HOME}/${SOURCE_P7B:2}"
elif [[ "$SOURCE_P7B" == "~" ]]; then
    SOURCE_P7B="$HOME"
fi

# 校验参数
if [[ -z "$SOURCE_P7B" ]]; then
    echo "用法: $0 <source_p7b> [branch]"
    echo ""
    echo "参数:"
    echo "  source_p7b  源 p7b 文件路径（必填）"
    echo "  branch      目标分支（默认: 自动获取最新 release 分支）"
    echo ""
    echo "示例:"
    echo "  $0 ~/Downloads/app/ajk-harmony-debugDebug\(1\).p7b"
    echo "  $0 ~/Downloads/app/ajk-hap-debug.p7b release-17.36"
    exit 1
fi

# 校验源文件
if [[ ! -f "$SOURCE_P7B" ]]; then
    echo "错误: 源文件不存在: $SOURCE_P7B"
    exit 1
fi

# 解析 branch 参数
if [[ -z "$BRANCH" ]]; then
    echo "未指定分支，自动获取最新 release 分支..."
    BRANCH=$(get_latest_release_branch)
    echo "最新 release 分支: $BRANCH"
fi

replace_with_glab
