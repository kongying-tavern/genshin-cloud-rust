# 域同步模板（Java → Rust）

> [← 返回索引](../README.md) · 全局进度见 [Java 同步路线图](./sync-with-java-roadmap.md)

从 Java 侧移植一个业务域到 Rust，需按下列**五层**依次落地。每一层都有固定位置与
命名约定，遵循约定后域与域之间结构一致、易于审查。以 `area` 域为贯穿示例。

## 五层模式

| 层 | 位置 | 内容 |
| --- | --- | --- |
| 1. 实体 | `packages/database/src/models/<domain>/` | sea-orm `Entity`/`Model`/`Column`，套用 `impl_safe_operation!` |
| 2. DTO/VO | `packages/utils/src/types/` 与 `src/models/` | 请求/响应结构（`XxxListRequest`、`XxxVO`、`CommonResponse`） |
| 3. 业务 | `packages/functions/src/functions/api/<domain>.rs` | `do_list` / `do_get` / `do_add` / `do_update` 等异步函数 |
| 4. 路由 | `packages/router/src/routes/api/<domain>/` | axum 处理函数 + `mod.rs` 注册 `Router` |
| 5. 测试 | `tests/rust/tests/<domain>/` | 端到端冒烟测试 |

## area 域迷你示例

**第 1 层 · 实体** `packages/database/src/models/area/area.rs`：

```rust
# [derive(Clone, Debug, DeriveEntityModel, Deserialize, Serialize)]
# [sea_orm(table_name = "area", schema_name = "genshin_map")]
pub struct Model {
    pub version: i64,                       // 乐观锁
    #[sea_orm(primary_key)] pub id: i64,
    pub del_flag: bool,                     // 软删除
    pub name: String,
    pub parent_id: i64,
    // ...其余字段
}
impl_safe_operation! { /* 列出 update_time/del_flag 列 */ }
```

**第 2 层 · DTO** `packages/utils/src/models/`：定义 `AreaListRequest`、
`AreaVO`、`AreaListResponse`，并以 `CommonResponse<T>` 统一包装。

**第 3 层 · 业务** `packages/functions/src/functions/api/area.rs`：

```rust
pub async fn do_list(auth: AuthInfo, payload: AreaListRequest)
    -> Result<CommonResponse<AreaListResponse>>
{
    let rows = area_model::Entity::find_safety()      // 自动过滤 del_flag
        .filter(area_model::Column::ParentId.eq(payload.parent_id))
        .all(&*DB_CONN).await?;
    // 映射为 VO、装进 CommonResponse
}
```

**第 4 层 · 路由** `packages/router/src/routes/api/area/list.rs` + `mod.rs`：

```rust
pub async fn list(ExtractAuthInfo(auth): ExtractAuthInfo,
                  Json(payload): Json<AreaListRequest>)
    -> Result<impl IntoResponse, (StatusCode, String)> {
    match _functions::functions::api::area::do_list(auth, payload).await {
        Ok(v) => Ok((StatusCode::OK, Json(v))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, format!("{e}"))),
    }
}
// mod.rs: .route("/get/list", post(list::list)) ...
```

**第 5 层 · 测试** `tests/rust/tests/area/area_domain_test.rs`：起一个真实（或
testcontainers）Postgres，调用路由做 list/get/add/update/delete 冒烟，校验软删除
与乐观锁自增。

## 检查清单

- [ ] 实体经 `impl_safe_operation!` 获得软删除 + 乐观锁
- [ ] DTO 在 `utils` 包，业务层不直接返回 sea-orm `Model`
- [ ] 路由统一用 `ExtractAuthInfo` + `CommonResponse`
- [ ] 冒烟测试覆盖至少 list/get/add/update/delete 五个路径
- [ ] commit subject 符合 gitmoji 规范
