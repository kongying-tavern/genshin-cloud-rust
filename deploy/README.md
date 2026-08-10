# 线上试部署说明（后端）

## 一、镜像（GitHub Actions 自动构建）

- 仓库：`ghcr.io/kongying-tavern/genshin-cloud-rust`
- master push → 自动构建并推送 `latest` + `sha-<短哈希>`
- 打 tag（`git tag v1.2.3 && git push --tags`）→ 额外推送 `1.2.3` / `1.2` / `1` / `latest`
- PR 只构建不推送

## 二、部署（目标服务器）

```bash
# 1. 准备（首次）
git clone https://github.com/langyo/genshin-cloud-rust.git   # 或仅拷贝 deploy/ 目录
cd deploy
cp .env.example .env
# 编辑 .env：DB_* / MINIO_* / JWT_SECRET / HOST_PORT 必填

# 2. 部署（默认 latest）
./deploy.sh

# 3. 回滚到指定构建
./deploy.sh sha-abc1234

# 4. 运维
docker compose ps
docker compose logs -f gcs-backend
docker compose restart gcs-backend
```

## 三、nginx 反代（可选，域名访问）

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

## 四、健康检查

- 容器 healthcheck：`GET /.well-known/jwks.json` 返回 200 即健康
- 手动验证：
  ```bash
  curl -s http://127.0.0.1:8101/.well-known/jwks.json
  curl -s -X POST http://127.0.0.1:8101/oauth/token \
    -F grant_type=password -F username=<user> -F password=<pass>
  ```

