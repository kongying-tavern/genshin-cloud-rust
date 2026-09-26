# tests/python — 活体 API 回归套件

对**本地起的后端**做全链路 HTTP/WS 回归（约 170+ 项断言），覆盖鉴权、基础数
据域、点位域、点位联动、doc 载荷、用户域与 WebSocket 广播。套件源自未入库的
`.regression/` 会话工件，收编时已脱敏（凭据一律走环境变量）并适配 WS 握手鉴
权（`GET /ws/{userId}?token=`，userId 与 token 主体必须一致）。

## 套件清单

| 套件 | 覆盖面 |
| --- | --- |
| `test_a_auth.py` | oauth/token 密码模式（正确/错误口令、不存在用户）、refresh_token 轮换与复用、client_credentials 访客态、jwks、网关层 401、五角色登录 |
| `test_b_base.py` | area / icon_type / icon（urlVariants 合并语义）/ item_type / item（copy、乐观锁）/ item_common 的 CRUD 与写权限矩阵 |
| `test_c_marker.py` | 点位建/查/改/删、tweak 批量替换、extra 特殊载荷往返、hiddenFlag 角色可见性矩阵 |
| `test_d_link.py` | 点位联动：建组、跨组合并、list 与 graph 一致性、按组删除、自环拒绝、权限门 |
| `test_e_docs.py` | `*_doc` md5 清单与 bin 载荷：gzip 解码、伪造 md5 拒绝、角色过滤分页与越权取页拒绝 |
| `test_g_user.py` | 用户信息可见性、自助改名、越权/提权拦截、改口令、注册、角色表、档案八件套、邀请码、公告、app 触发器、res 探测、缓存清理 |
| `test_ws_res.py` | WS 握手鉴权（无 token 401、冒用 userId 403）、MarkerAdded/NoticeAdded/BinaryPurged 广播、res 图片上传（依赖本地 MinIO） |

各套件创建的数据一律以 `regress` 前缀命名，并在套件结尾清理（清理本身也
在验证递归删除等路径）。

## 运行前置

1. **数据库**：`docker compose -f tests/docker/docker-compose.e2e.yml up -d`
   起本地 Postgres + MinIO + Redis，或把后端指向 dev 库。注意：少数断言
   依赖 dev 量级的数据形状（如 beta 专属 doc 分页、marker_doc 多分页、
   area 列表非空），全新空库上会以「数据不足」的形式失败——属预期。
2. **后端**：本地启动（默认监听 `http://127.0.0.1:8101`）。
3. **种子账号**：库中须预先存在 6 个 `regress_*` 账号，口令一致且等于
   `REGRESSION_PASSWORD`：

   | 账号 | role |
   | --- | --- |
   | `regress_admin` | admin (0) |
   | `regress_manager` | MAP_MANAGER (1) |
   | `regress_beta` | MAP_NEIGUI (2) |
   | `regress_punctuate` | MAP_PUNCTUATE (3) |
   | `regress_user` | MAP_USER (4) |
   | `regress_newuser` | MAP_USER (4)，注册重复名断言的目标账号 |

   `regress_newuser` 供 `test_g_user` 的注册断言使用：套件首次运行会尝试
   注册它，已存在时断言按「already exists」宽容通过，两种状态都合法。

   目前仓库没有 seed 脚本入口；账号由运维手工或既有 seed 流程创建，口令
   写法可参考（`password` 列为 bcrypt 密文，与 `REGRESSION_PASSWORD` 对应）：

   ```sql
   -- 示意：role_id 与 username 一一对应，见上表
   INSERT INTO genshin_map.sys_user (username, password, nickname, role_id)
   VALUES ('regress_admin', '<bcrypt(REGRESSION_PASSWORD)>', 'regress admin', 0);
   -- …… regress_manager=1 / regress_beta=2 / regress_punctuate=3 /
   --       regress_user=4 / regress_newuser=4
   ```

## 环境变量

| 变量 | 说明 |
| --- | --- |
| `REGRESSION_BASE` | 后端地址，默认 `http://127.0.0.1:8101` |
| `REGRESSION_PASSWORD` | **必设**。regress_* 六账号的统一口令 |

**为何凭据走环境变量**：本套件曾对共享开发库运行，该口令属真实凭据。
按仓库敏感信息红线（AGENTS.md §8），任何真实口令/密钥一律禁止入库，因此
`common.PASSWORD` 只从 `REGRESSION_PASSWORD` 读取、无默认值；未设置时各套
件打印 `skipped: set REGRESSION_PASSWORD` 并以 0 退出（自跳过）。

## 运行方式

```bash
# 全量（自动发现本目录全部 test_*.py，按文件名排序执行）
REGRESSION_PASSWORD=... python tests/python/run_all.py

# 单套件
REGRESSION_PASSWORD=... python tests/python/test_a_auth.py
```

依赖：Python 3.10+ 标准库即可；`test_ws_res.py` 额外需要
`pip install "websockets>=14"`（未安装时该套件自跳过）。
