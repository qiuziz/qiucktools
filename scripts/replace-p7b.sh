#!/bin/bash
# replace-p7b.sh - 替换 harmonyajkproject 中的签名文件并提交 MR
# 用法: ./replace-p7b.sh <source_p7b> [branch]
# 或者: ./replace-p7b.sh [key value]... （key-value 模式，由 QuickTools executor 调用）
#
# 示例:
#   ./replace-p7b.sh ~/Downloads/app/ajk-harmony-debugDebug\(1\).p7b
#   ./replace-p7b.sh ~/Downloads/app/ajk-hap-debug.p7b release-17.36
#
# 参数说明:
#   source_p7b  源 p7b 文件路径（必填）
#   branch      目标分支（默认: 最新 release 分支）
#
# 流程:
#   1. 获取最新 release 分支
#   2. 创建 feature 分支
#   3. 替换签名文件并提交
#   4. 推送到远程并创建 MR
#   5. 自动合并 MR

set -euo pipefail

# QuickTools GUI 启动时不继承 shell PATH，需显式补充
export PATH="/usr/local/bin:/opt/homebrew/bin:$HOME/bin:$HOME/.local/bin:$PATH"

# ---------- 固定配置 ----------
GITLAB_HOST="igit.58corp.com"
GITLAB_PROJECT="_fe%2Fharmonyajkproject"
REPO_DIR="/Users/qiuz/work/AJK/harmony/harmonyajkproject"
REPO_FILE="config/debugSign/ajk-hap-debug.p7b"
COMMIT_MSG="chore: replace ajk-hap-debug.p7b"
# ------------------------------

log_info() {
    echo "[INFO] $*" >&2
}

log_error() {
    echo "[ERROR] $*" >&2
}

# 解析参数：支持两种模式
# 1. 传统模式: $1=path, $2=branch
# 2. Key-value 模式: $1=key1, $2=val1, $3=key2, $4=val2, ...
parse_args() {
    local source_p7b=""
    local branch=""

    if [[ $# -ge 2 ]] && [[ "$1" == "sourceP7b" || "$1" == "branch" ]]; then
        # Key-value 模式
        while [[ $# -ge 2 ]]; do
            case "$1" in
                sourceP7b) source_p7b="$2" ;;
                branch) branch="$2" ;;
            esac
            shift 2
        done
    else
        # 传统模式
        source_p7b="${1:-}"
        branch="${2:-}"
    fi

    # 返回值通过全局变量
    SOURCE_P7B_GLOBAL="$source_p7b"
    BRANCH_GLOBAL="$branch"
}

require_cmd() {
    local cmd="$1"
    if ! command -v "$cmd" > /dev/null 2>&1; then
        log_error "缺少命令: $cmd"
        exit 1
    fi
}

# 检查源文件是否存在
check_source_file() {
    local file="$1"

    # 检查是否是占位符（参数未替换）
    if [[ "$file" == *"{{"* ]] || [[ "$file" == *"}}"* ]]; then
        log_error "参数未正确替换，检测到占位符: $file"
        log_error "请确保所有参数都已填写"
        exit 1
    fi

    # 展开 ~
    if [[ "${file:0:2}" == "~/" ]]; then
        file="${HOME}/${file:2}"
    elif [[ "$file" == "~" ]]; then
        file="$HOME"
    fi

    if [[ ! -f "$file" ]]; then
        log_error "源文件不存在: $file"
        exit 1
    fi

    echo "$file"
}

# 获取最新 release 分支
get_latest_release_branch() {
    require_cmd glab
    require_cmd jq

    local branches
    branches=$(glab api --hostname "$GITLAB_HOST" "projects/${GITLAB_PROJECT}/repository/branches?search=release-&per_page=100" --paginate 2>/dev/null \
        | jq -r '.[].name' \
        | grep '^release-' \
        | sort -t'.' -k1,1n -k2,2n -k3,3n \
        | tail -1)

    if [[ -z "$branches" ]]; then
        log_error "无法获取 release 分支列表"
        exit 1
    fi

    echo "$branches"
}

# ============ 主流程 ============
main() {
    log_info "=== 开始替换 P7B 签名文件 ==="

    # 解析参数
    parse_args "$@"
    local source_p7b="$SOURCE_P7B_GLOBAL"
    local target_branch="$BRANCH_GLOBAL"

    # 检查源文件
    source_p7b=$(check_source_file "$source_p7b")
    log_info "源文件: $source_p7b"

    # 检查仓库目录
    if [[ ! -d "$REPO_DIR" ]]; then
        log_error "仓库目录不存在: $REPO_DIR"
        exit 1
    fi

    cd "$REPO_DIR" || {
        log_error "无法进入目录: $REPO_DIR"
        exit 1
    }

    # 获取最新 release 分支
    if [[ -z "$target_branch" ]] || [[ "$target_branch" == "''" ]]; then
        target_branch=$(get_latest_release_branch)
    fi
    log_info "目标分支: $target_branch"

    # 生成 feature 分支名
    local feature_branch="feature/replace-p7b-$(date +%Y%m%d%H%M)"
    log_info "创建 feature 分支: $feature_branch (基于 $target_branch)"

    # 检查是否有未提交的更改
    if ! git diff-index --quiet HEAD -- 2>/dev/null; then
        log_info "存在未提交的更改，先暂存..."
        git stash push -m "temp stash before replace-p7b"
    fi

    # 获取远程分支并清理可能的本地分支
    git fetch origin "$target_branch" --quiet 2>/dev/null
    git branch -D "$feature_branch" 2>/dev/null || true

    # 创建新分支
    if ! git checkout -b "$feature_branch" "origin/$target_branch" 2>&1; then
        log_error "创建分支失败"
        exit 1
    fi

    log_info "复制文件到工作目录..."
    cp -f "$source_p7b" "$REPO_FILE"

    log_info "提交更改..."
    git add "$REPO_FILE"

    # 检查是否有更改需要提交
    if git status --porcelain | grep -q .; then
        if ! git commit -m "$COMMIT_MSG"; then
            log_error "提交失败"
            exit 1
        fi

        log_info "推送到远程..."
        if ! git push -u origin "$feature_branch" 2>&1; then
            log_error "推送失败"
            exit 1
        fi
    else
        log_info "文件内容未变化，跳过提交"
    fi

    # 创建 MR
    log_info "创建 MR..."
    local mr_url
    mr_url=$(glab api --hostname "$GITLAB_HOST" "projects/${GITLAB_PROJECT}/merge_requests" \
        --method POST \
        --field "source_branch=$feature_branch" \
        --field "target_branch=$target_branch" \
        --field "title=$COMMIT_MSG" \
        --field "description=自动替换签名文件" \
        --field "squash=false" 2>&1 | jq -r '.web_url // empty')

    if [[ -z "$mr_url" ]]; then
        log_error "创建 MR 失败"
        exit 1
    fi

    log_info "MR 地址: $mr_url"

    # 提取 MR 号
    local mr_iid
    mr_iid=$(echo "$mr_url" | grep -oE '[0-9]+$' | tail -1)

    log_info "合并 MR #${mr_iid}..."
    local merge_result
    merge_result=$(glab api --hostname "$GITLAB_HOST" --method PUT "projects/${GITLAB_PROJECT}/merge_requests/${mr_iid}/merge" 2>&1)
    local merge_status
    merge_status=$(echo "$merge_result" | jq -r '.state // "unknown"')
    if [[ "$merge_status" != "merged" ]]; then
        log_error "合并 MR 失败: $merge_result"
        exit 1
    fi

    log_info "MR 已合并!"

    # 清理：切换回 release 分支
    git checkout "$target_branch" 2>/dev/null || git checkout "origin/$target_branch"
    git branch -D "$feature_branch" 2>/dev/null || true

    log_info ""
    log_info "=== 完成 ==="
    log_info "签名文件已替换并合并到 $target_branch"
    log_info "MR: $mr_url"
}

main "$@"