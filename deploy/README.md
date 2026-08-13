# 线上试部署说明（后端）

## 一、获取部署文件

生产服务器通常不需要整个仓库（源码、11 种语言文档），只需 `deploy/` 目录即可。
本目录提供两种方式：

### 方式 A：稀疏检出（推荐，只需 deploy/）

```bash
# 服务器上生成 deploy/ 目录
curl -fsSL https://raw.githubusercontent.com/kongying-tavern/genshin-cloud-rust/master/deploy/fetch.sh -o fetch.sh
bash fetch.sh deploy        # 默认 GCS_REPO_URL=.../kongying-tavern/genshin-cloud-rust，GCS_BRANCH=master
cd deploy
cp .env.example .env
# 编辑 .env：DB_* / MINIO_* / JWT_SECRET / HOST_PORT 必填
```

`fetch.sh` 使用 `git clone --filter=blob:none --sparse` + `git sparse-checkout set deploy`
只拉取 `deploy/` 目录（blob 按需下载），不下载 `packages/`、`docs/` 等无关内容。
需 git >= 2.25。可覆盖 `GCS_REPO_URL` / `GCS_BRANCH` 指定其他仓库/分支。

更新时重新执行 `bash fetch.sh deploy` 即可覆盖为最新版本。

### 方式 B：全仓克隆（开发机习惯）

```bash
git clone https://github.com/kongying-tavern/genshin-cloud-rust.git
cd genshin-cloud-rust/deploy
cp .env.example .env
# 编辑 .env：DB_* / MINIO_* / JWT_SECRET / HOST_PORT 必填
```

## 二、镜像（GitHub Actions 自动构建）

- 仓库：`ghcr.io/kongying-tavern/genshin-cloud-rust`
- master push → 自动构建并推送 `latest` + `sha-<短哈希>`
- 打 tag（`git tag v1.2.3 && git push --tags`）→ 额外推送 `1.2.3` / `1.2` / `1` / `latest`
- PR 只构建不推送

## 三、部署（目标服务器）

在 `deploy/` 目录内执行：

```bash
# 1. 部署（默认 latest）
./deploy.sh

# 2. 回滚到指定构建
./deploy.sh sha-abc1234

# 3. 运维
docker compose ps
docker compose logs -f gcs-backend
docker compose restart gcs-backend
```

## 四、nginx 反代（可选，域名访问）

```nginx
server {
    listen 443 ssl;
    server_name map.example.com;
    # ... ssl 配置 ...

    location / {
        proxy_pass http://127.0.0.1:8101;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $remote_addr;
        proxy_set_header X-Forwarded-Proto $scheme;
        # WebSocket 升级（公告/点位实时推送）
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 60s;
    }
}
```

> 使用 nginx 反代时：`.env` 里 `HOST_PORT=127.0.0.1:8101`，并开启 `TRUST_PROXY_HEADERS=true`（否则 access_policy 的 IP 判定会拿到反代地址）。

## 五、健康检查

- 容器 healthcheck：`GET /.well-known/jwks.json` 返回 200 即健康
- 手动验证：
  ```bash
  curl -s http://127.0.0.1:8101/.well-known/jwks.json
  curl -s -X POST http://127.0.0.1:8101/oauth/token \
    -F grant_type=password -F username=<user> -F password=<pass>
  ```