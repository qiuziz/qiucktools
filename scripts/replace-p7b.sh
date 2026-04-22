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
#   1. 检查目标目录是否存在，不存在则从 git clone
#   2. 切换到目标分支
#   3. 备份原文件（*.p7b.bak）
#   4. 替换文件
#   5. git add + commit + push

set -e

SOURCE_P7B="${1:-}"
BRANCH="${2:-}"

# ---------- 固定配置 ----------
readonly PROJECT_NAME="harmonyajkproject"
readonly GIT_URL="git@igit.58corp.com:_fe/harmonyajkproject.git"
readonly GIT_DIR="/Users/qiuz/work/AJK/harmony/$PROJECT_NAME"
readonly TARGET_FILE="$GIT_DIR/config/debugSign/ajk-hap-debug.p7b"
readonly COMMIT_MSG="chore: replace ajk-hap-debug.p7b"
# ------------------------------

# 获取最新 release 分支（按版本号降序排列，取第一个）
get_latest_release_branch() {
    local branches
    branches=$(git ls-remote --heads "$GIT_URL" 2>/dev/null | awk -F'/' '{print $NF}' | grep '^release-' | sort -t'.' -k1,1n -k2,2n -k3,3n | tail -1)
    if [[ -z "$branches" ]]; then
        echo "release-17.36"  # fallback
    else
        echo "$branches"
    fi
}

# 展开 ~（不用 eval，避免路径含特殊字符时出错）
if [[ "$SOURCE_P7B" == ~/* ]]; then
    SOURCE_P7B="${HOME}/${SOURCE_P7B:2}"
elif [[ "$SOURCE_P7B" == "~" ]]; then
    SOURCE_P7B="$HOME"
fi

# 解析 branch 参数
if [[ -z "$BRANCH" ]]; then
    echo "未指定分支，自动获取最新 release 分支..."
    BRANCH=$(get_latest_release_branch)
    echo "最新 release 分支: $BRANCH"
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

# ---------- 中断清理 ----------
cleanup() {
    if [[ "$NEED_CLEANUP" == true ]] && [[ -d "$GIT_DIR" ]]; then
        echo ""
        echo "中断：清理临时克隆目录..."
        rm -rf "$GIT_DIR"
        echo "已删除: $GIT_DIR"
    fi
    rm -f "${TARGET_FILE}.bak"
}
trap cleanup EXIT INT TERM

# ---------- 检查/克隆项目 ----------
NEED_CLEANUP=false
if [[ ! -d "$GIT_DIR" ]]; then
    echo "目标目录不存在: $GIT_DIR"
    echo "从远程克隆..."
    git clone "$GIT_URL" "$GIT_DIR"
    echo "克隆完成"
    NEED_CLEANUP=true
fi

# 切换到目标目录
cd "$GIT_DIR"

# 检查 git 状态
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo "错误: $GIT_DIR 不是 git 仓库"
    exit 1
fi

# 拉取最新分支列表
echo "更新远程分支列表..."
git fetch origin > /dev/null 2>&1

# 检查分支是否存在，不存在则创建
if ! git rev-parse --verify "$BRANCH" > /dev/null 2>&1; then
    echo "本地分支 $BRANCH 不存在，创建并切换..."
    git checkout -b "$BRANCH" "origin/$BRANCH" 2>/dev/null || {
        echo "错误: 远程分支 origin/$BRANCH 不存在"
        exit 1
    }
else
    git checkout "$BRANCH"
fi

# 拉取最新代码
echo "拉取最新代码..."
git pull origin "$BRANCH" --ff > /dev/null 2>&1 || echo "（拉取可能失败，如有问题请手动检查）"

# ---------- 替换文件 ----------
ORIG_SIZE=$(stat -f%z "$TARGET_FILE" 2>/dev/null || stat -c%s "$TARGET_FILE" 2>/dev/null || echo "unknown")
NEW_SIZE=$(stat -f%z "$SOURCE_P7B" 2>/dev/null || stat -c%s "$SOURCE_P7B" 2>/dev/null || echo "unknown")

echo ""
echo "=== 开始替换签名文件 ==="
echo "源文件:   $SOURCE_P7B"
echo "目标文件: $TARGET_FILE"
echo "分支:     $BRANCH"
echo "原文件:   $ORIG_SIZE bytes"
echo "新文件:   $NEW_SIZE bytes"

# 备份原文件
cp "$TARGET_FILE" "${TARGET_FILE}.bak"
echo "已备份原文件到: ${TARGET_FILE}.bak"

# 替换文件
cp "$SOURCE_P7B" "$TARGET_FILE"
echo "文件替换完成"

# 检查 git diff（Binary files 没有 insertions/deletions 统计，用 --quiet 判断）
if git diff --quiet config/debugSign/ajk-hap-debug.p7b; then
    echo ""
    echo "警告: 文件内容未变化，跳过提交"
    exit 0
fi

# git add
git add config/debugSign/ajk-hap-debug.p7b

# git commit
echo ""
echo "提交变更..."
git commit -m "$COMMIT_MSG"
COMMIT_HASH=$(git rev-parse --short HEAD)
echo "已提交: $COMMIT_HASH"

# git push
echo ""
echo "推送到远程..."
git push origin "$BRANCH"
echo ""
echo "=== 完成 ==="
echo "分支: $BRANCH"
echo "提交: $COMMIT_HASH"
