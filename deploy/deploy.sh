#!/usr/bin/env bash
# ============ 空荧酒馆后端线上部署脚本（试部署） ============
#
# 前置：
#   1. 本机已安装 docker + docker compose v2
#   2. 已配置 GHCR 拉取凭证（可选：私有仓库需要；公开仓库无需登录）
#      export GHCR_USER=langyo
#      export GHCR_TOKEN=<PAT with read:packages>
#
# 用法：
#   ./deploy.sh [镜像tag]      # 默认 latest；可指定 sha-xxxxx / v1.2.3 回滚
#
# 说明：本脚本在 deploy/ 目录执行；.env 与本脚本同目录。

set -euo pipefail

cd "$(dirname "$0")"

TAG="${1:-latest}"
IMAGE="ghcr.io/kongying-tavern/genshin-cloud-rust:${TAG}"
ENV_FILE="${ENV_FILE:-.env}"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "[ERROR] 缺少 ${ENV_FILE}，请先 cp .env.example .env 并填写配置" >&2
  exit 1
fi

if [[ -n "${GHCR_TOKEN:-}" ]]; then
  echo "[INFO] 登录 GHCR ..."
  echo "$GHCR_TOKEN" | docker login ghcr.io -u "${GHCR_USER:?GHCR_USER 未设置}" --password-stdin
fi

echo "[INFO] 拉取镜像 ${IMAGE} ..."
docker pull "$IMAGE"

echo "[INFO] 部署 ${IMAGE} ..."
if docker compose ps -q gcs-backend >/dev/null 2>&1 && [[ "$TAG" != "latest" ]]; then
  # 显式指定非 latest 镜像：通过 IMAGE_TAG 覆盖 compose 默认的 latest
  IMAGE_TAG="$TAG" docker compose up -d --no-deps gcs-backend
else
  docker compose up -d
fi

echo "[INFO] 等待健康检查 ..."
HOST_PORT="$(grep -E '^HOST_PORT=' "$ENV_FILE" | cut -d= -f2)"
HOST_PORT="${HOST_PORT:-8101}"
for i in $(seq 1 30); do
  if docker compose ps gcs-backend | grep -q healthy; then
    echo "[OK] gcs-backend 已就绪：http://127.0.0.1:${HOST_PORT}"
    echo "     健康检查：curl -s http://127.0.0.1:${HOST_PORT}/.well-known/jwks.json"
    echo "     日志：docker compose logs -f gcs-backend"
    exit 0
  fi
  sleep 5
done

echo "[WARN] 等待超时，当前状态：" >&2
docker compose ps >&2
exit 1

