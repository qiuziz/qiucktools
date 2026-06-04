#!/bin/bash
# replace-hbc.sh - 替换 harmonyajkproject 中的 mainhouse.hbc 并提交 MR
# 用法: ./replace-hbc.sh <source_hbc> [branch]
# 或者: ./replace-hbc.sh [key value]... （key-value 模式，由 QuickTools executor 调用）

set -euo pipefail

export PATH="/usr/local/bin:/opt/homebrew/bin:$HOME/bin:$HOME/.local/bin:$PATH"

GITLAB_HOST="igit.58corp.com"
GITLAB_PROJECT="_fe%2Fharmonyajkproject"
REPO_DIR="/Volumes/ajk-disk/work/AJK/harmonyajkproject"
REPO_FILE="entry/src/main/resources/rawfile/rn/1875/mainhouse.hbc"
COMMIT_MSG="feat: 更新 mainhouse.hbc 资源文件"
FEATURE_BRANCH="feature/update-mainhouse-hbc-$(date +%Y%m%d%H%M)"

log_info() { echo "[INFO] $*" >&2; }
log_error() { echo "[ERROR] $*" >&2; }

parse_args() {
    local source_hbc="" branch=""
    if [[ $# -ge 2 ]] && [[ "$1" == "sourceHbc" || "$1" == "branch" ]]; then
        while [[ $# -ge 2 ]]; do
            case "$1" in sourceHbc) source_hbc="$2" ;; branch) branch="$2" ;; esac
            shift 2
        done
    else
        source_hbc="${1:-}" branch="${2:-}"
    fi
    SOURCE_HBC_GLOBAL="$source_hbc"
    BRANCH_GLOBAL="$branch"
}

require_cmd() {
    if ! command -v "$1" > /dev/null 2>&1; then
        log_error "缺少命令: $1"; exit 1
    fi
}

check_source_file() {
    local file="$1"
    if [[ "$file" == *"{{"* ]] || [[ "$file" == *"}}"* ]]; then
        log_error "参数未正确替换: $file"; exit 1
    fi
    if [[ ! -f "$file" ]]; then
        log_error "源文件不存在: $file"; exit 1
    fi
    echo "$file"
}

get_latest_release_branch() {
    require_cmd glab; require_cmd jq
    local branches
    branches=$(glab api --hostname "$GITLAB_HOST" "projects/${GITLAB_PROJECT}/repository/branches?search=release-&per_page=100" --paginate 2>/dev/null \
        | jq -r '.[].name' | grep '^release-' | sort -t'.' -k1,1n -k2,2n -k3,3n | tail -1)
    if [[ -z "$branches" ]]; then
        log_error "无法获取 release 分支列表"; exit 1
    fi
    echo "$branches"
}

main() {
    log_info "=== 开始替换 mainhouse.hbc ==="

    parse_args "$@"
    local source_hbc="$SOURCE_HBC_GLOBAL"
    local target_branch="$BRANCH_GLOBAL"

    source_hbc=$(check_source_file "$source_hbc")
    log_info "源文件: $source_hbc"

    if [[ ! -d "$REPO_DIR" ]]; then
        log_error "仓库目录不存在: $REPO_DIR"; exit 1
    fi

    cd "$REPO_DIR" || { log_error "无法进入目录: $REPO_DIR"; exit 1; }

    if [[ -z "$target_branch" ]] || [[ "$target_branch" == "''" ]]; then
        target_branch=$(get_latest_release_branch)
    fi
    log_info "目标分支: $target_branch"

    log_info "创建 feature 分支: $FEATURE_BRANCH (基于 $target_branch)"

    git fetch origin "$target_branch" --quiet 2>/dev/null
    git branch -D "$FEATURE_BRANCH" 2>/dev/null || true

    # 暂存本地所有改动（避免 checkout 时冲突）
    git stash push --include-untracked -m "temp stash before replace-hbc" 2>/dev/null || true

    if ! git checkout -b "$FEATURE_BRANCH" "origin/$target_branch" 2>&1; then
        log_error "创建分支失败"; exit 1
    fi

    log_info "复制文件到工作目录..."
    cp -f "$source_hbc" "$REPO_FILE"

    log_info "提交更改..."
    git add "$REPO_FILE"

    if git status --porcelain | grep -q .; then
        if ! git commit -m "$COMMIT_MSG"; then
            log_error "提交失败"; exit 1
        fi
        log_info "推送分支..."
        if ! git push -u origin "$FEATURE_BRANCH" 2>&1; then
            log_error "推送失败"; exit 1
        fi
    else
        log_info "文件内容未变化，跳过提交"; git checkout "$target_branch" 2>/dev/null; exit 0
    fi

    log_info "创建 MR..."
    local mr_url
    mr_url=$(glab api --hostname "$GITLAB_HOST" "projects/${GITLAB_PROJECT}/merge_requests" \
        --method POST \
        --field "source_branch=$FEATURE_BRANCH" \
        --field "target_branch=$target_branch" \
        --field "title=$COMMIT_MSG" \
        --field "description=自动更新 mainhouse.hbc 资源文件" \
        --field "squash=false" 2>&1 | jq -r '.web_url // empty')

    if [[ -z "$mr_url" ]]; then
        log_error "创建 MR 失败"; exit 1
    fi

    log_info "MR 地址: $mr_url"
    echo "$mr_url"
}

main "$@"