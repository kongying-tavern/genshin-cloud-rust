#!/usr/bin/env bash
# ============ 空荧酒馆后端 — 稀疏检出部署脚本 ============
#
# 只在目标服务器上检出仓库的 deploy/ 目录（不含 packages/、docs/ 等源码），
# 用于「只部署、不开发」的服务器环境，避免拉取整个仓库。
#
# 用法：
#   ./fetch.sh [target-dir]      # 默认在 ./deploy 生成部署文件
#
# 环境变量覆盖：
#   GCS_REPO_URL   仓库地址（默认 kongying-tavern/genshin-cloud-rust）
#   GCS_BRANCH     分支（默认 master）
#
# 依赖：git >= 2.25（支持 --filter=blob:none + sparse-checkout cone 模式）。
#
# 之后照常部署：
#   cd deploy
#   cp .env.example .env   # 编辑配置
#   ./deploy.sh

set -euo pipefail

REPO_URL="${GCS_REPO_URL:-https://github.com/kongying-tavern/genshin-cloud-rust.git}"
BRANCH="${GCS_BRANCH:-master}"
TARGET="${1:-deploy}"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "[INFO] 稀疏检出 deploy/ from ${REPO_URL} (branch ${BRANCH}) ..."
git clone --depth 1 --filter=blob:none --sparse -b "$BRANCH" "$REPO_URL" "$TMP/repo"
git -C "$TMP/repo" sparse-checkout set deploy

echo "[INFO] 复制部署文件到 ${TARGET} ..."
mkdir -p "$TARGET"
cp -a "$TMP/repo/deploy/." "$TARGET/"

echo "[OK] 部署文件已就绪：${TARGET}"
echo "     下一步：cd ${TARGET} && cp .env.example .env && ./deploy.sh"
