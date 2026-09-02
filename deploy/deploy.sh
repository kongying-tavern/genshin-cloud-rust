#!/usr/bin/env bash
# ============ 空荧酒馆后端线上部署脚本 ============
#
# 用法：
#   ./deploy.sh [tag]             # 默认 latest；可指定 sha-xxxxx / v1.2.3 回滚
#   ./deploy.sh [tag] --force     # 强制重建容器（即使镜像 digest 未变化）
#
# 环境变量：
#   ENV_FILE=path                 # .env 文件路径（默认脚本同目录 .env）
#   GHCR_USER / GHCR_TOKEN        # 私有仓库拉取凭证（公开仓库无需登录）
#
# 说明：
#   1. 总是先 docker pull（latest 保持最新；固定 tag 失败即中止）
#   2. 容器已健康且镜像 digest 未变化时跳过重建（--force 可强制）
#   3. 启动后等待健康检查（30 x 5s）；失败时输出日志并尝试回滚
#   4. 通过 IMAGE_TAG 把固定 tag 传给 compose（回滚/固定版本）
set -euo pipefail

cd "$(dirname "$0")"

TAG="${1:-latest}"
FORCE=0
if [[ "${2:-}" == "--force" ]]; then FORCE=1; fi
IMAGE="ghcr.io/kongying-tavern/genshin-cloud-rust:${TAG}"
ENV_FILE="${ENV_FILE:-.env}"
LAST_GOOD_FILE="/var/lib/genshin-cloud-deploy/last-good-tag"

if [[ ! -f "$ENV_FILE" ]]; then
  echo "[ERROR] 缺少 ${ENV_FILE}，请先 cp .env.example .env 并填写配置" >&2
  exit 1
fi

if [[ -n "${GHCR_TOKEN:-}" ]]; then
  echo "[INFO] 登录 GHCR ..."
  echo "$GHCR_TOKEN" | docker login ghcr.io -u "${GHCR_USER:?GHCR_USER 未设置}" --password-stdin
fi

echo "[INFO] 拉取镜像 ${IMAGE} ..."
docker pull "$IMAGE" 2>&1 | tail -2

if [[ "$FORCE" != "1" ]] && docker compose ps -q gcs-backend >/dev/null 2>&1; then
  CUR_IMG_ID=$(docker inspect --format '{{.Image}}' gcs-backend 2>/dev/null || true)
  NEW_IMG_ID=$(docker inspect --format '{{.Id}}' "$IMAGE" 2>/dev/null || true)
  if [[ -n "$CUR_IMG_ID" && "$CUR_IMG_ID" == "$NEW_IMG_ID" ]] && docker compose ps gcs-backend | grep -q healthy; then
    echo "[INFO] 镜像 digest 未变化且服务健康，跳过重建。"
    exit 0
  fi
fi

echo "[INFO] 部署 ${IMAGE} ..."
if docker compose ps -q gcs-backend >/dev/null 2>&1 && [[ "$TAG" != "latest" ]]; then
  # 固定 tag：通过 IMAGE_TAG 覆盖 compose 默认的 latest，只重建 gcs-backend
  IMAGE_TAG="$TAG" docker compose up -d --no-deps gcs-backend
else
  docker compose up -d
fi

echo "[INFO] 等待健康检查 ..."
HOST_PORT="$(grep -E '^HOST_PORT=' "$ENV_FILE" | cut -d= -f2)"
HOST_PORT="${HOST_PORT:-8101}"
ok=0
for _ in $(seq 1 30); do
  if docker compose ps gcs-backend | grep -q healthy; then
    echo "[OK] gcs-backend 已就绪：http://127.0.0.1:${HOST_PORT}"
    echo "     健康检查：curl -s http://127.0.0.1:${HOST_PORT}/.well-known/jwks.json"
    echo "     日志：docker compose logs -f gcs-backend"
    mkdir -p "$(dirname "$LAST_GOOD_FILE")"
    echo "$TAG" > "$LAST_GOOD_FILE"
    ok=1
    break
  fi
  sleep 5
done

if [[ "$ok" != "1" ]]; then
  echo "[WARN] 健康检查超时，当前状态：" >&2
  docker compose ps >&2
  echo "--- gcs-backend 最近日志 ---" >&2
  docker compose logs --tail 30 gcs-backend >&2 || true
  LAST_GOOD="$(cat "$LAST_GOOD_FILE" 2>/dev/null || true)"
  if [[ -n "$LAST_GOOD" && "$LAST_GOOD" != "$TAG" ]]; then
    echo "[INFO] 回滚到上次可用版本 ${LAST_GOOD} ..." >&2
    docker pull "ghcr.io/kongying-tavern/genshin-cloud-rust:${LAST_GOOD}" >/dev/null 2>&1 || true
    IMAGE_TAG="$LAST_GOOD" docker compose up -d --no-deps gcs-backend >&2 || true
    echo "[WARN] 已尝试回滚，请人工确认。" >&2
  else
    echo "[WARN] 无可用回滚版本（首次部署或 latest 本身失败）。" >&2
  fi
  exit 1
fi
