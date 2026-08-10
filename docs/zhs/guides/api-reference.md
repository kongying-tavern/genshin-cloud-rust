# API 参考概览

> [← 返回索引](../README.md) · 路由源码见 `packages/router/src/routes/`

本页列出 `_router` 二进制（axum）暴露的全部 API 域，按用途分组。这些端点均为
Java 侧控制器的 Rust 移植：路径、请求/响应结构与 Java 参考实现保持一致，仅
实现语言与传输细节不同。每个域对应 `functions/src/functions/api/<domain>.rs`
的业务函数与 `router/src/routes/api/<domain>/` 下的路由文件。

## 地图内容域

| 域 | 路径前缀 | 说明 |
| --- | --- | --- |
| area | `/area` | 地区树（含父子关系、末端标记、权限屏蔽） |
| icon | `/icon` | 地图图标资源及其元数据 |
| icon_type | `/icon_type` | 图标分类 |
| item | `/item` | 地图条目（item），含 copy/join 等聚合操作 |
| item_type | `/item_type` | 条目分类，含 move_type 排序 |
| item_common | `/itemCommon` | 条目公共属性 |
| marker | `/marker` | 打点（marker）的增删改查、single、tweak |
| marker_link | `/marker_link` | 打点之间的关联关系 |
| notice | `/notice` | 公告管理 |
| route | `/route` | 路线（route）管理 |
| history | `/history` | 历史版本列表 |
| cache | `/cache` | 按域的缓存刷新（area/item/marker/icon_tag/notice 等） |

## 归档与文档导出

| 域 | 路径前缀 | 说明 |
| --- | --- | --- |
| item_doc | `/itemDoc` | 条目分页导出（bin 二进制 / md5 校验） |
| marker_doc | `/markerDoc` | 打点分页导出（bin / md5） |
| marker_link_doc | `/marker_link_doc` | 打点关联导出 |
| res | `/res` | 资源上传（MinIO） |

> `*_doc` 系列对应 Java 侧 BinaryMD5 压缩归档导出能力。

## 打点审批与评分

| 域 | 路径前缀 | 说明 |
| --- | --- | --- |
| punctuate | `/punctuate` | 用户提交打点（actions / get / manage） |
| punctuate_audit | `/punctuate_audit` | 打点审批流（audit / delete / get） |
| score | `/score` | 评分数据与生成 |

## 系统域

挂在 `/system` 下（`router/src/routes/system/`）：

| 域 | 说明 |
| --- | --- |
| user | 用户管理 |
| role | 角色与权限 |
| device | 设备登记 |
| invitation | 邀请码 |
| action_log | 操作日志 |
| oauth | OAuth2 接入 |
| archive | 归档管理 |

## 鉴权

除少量公开端点外，所有路由经 `ExtractAuthInfo` 中间件提取 JWT 中的 `AuthInfo`，
业务函数据此做权限判定。鉴权细节见 `packages/utils/src/jwt.rs`。
