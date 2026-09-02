# 域同步模板（Java → Rust）

> [← 返回索引](../README.md) · 全局進度見 [Java 同步路線圖](./sync-with-java-roadmap.md)

從 Java 側移植一個業務域到 Rust，需按下列**五層**依次落地。每一層都有固定位置與
命名約定，遵循約定後域與域之間結構一致、易於審查。以 `area` 域爲貫穿示例。

## 五層模式

| 層 | 位置 | 內容 |
| --- | --- | --- |
| 1. 實體 | `packages/database/src/models/<domain>/` | sea-orm `Entity`/`Model`/`Column`，套用 `impl_safe_operation!` |
| 2. DTO/VO | `packages/utils/src/types/` 與 `src/models/` | 請求/響應結構（`XxxListRequest`、`XxxVO`、`CommonResponse`） |
| 3. 業務 | `packages/functions/src/functions/api/<domain>.rs` | `do_list` / `do_get` / `do_add` / `do_update` 等異步函數 |
| 4. 路由 | `packages/router/src/routes/api/<domain>/` | axum 處理函數 + `mod.rs` 註冊 `Router` |
| 5. 測試 | `tests/rust/tests/<domain>/` | 端到端冒煙測試 |

## area 域迷你示例

**第 1 層 · 實體** `packages/database/src/models/area/area.rs`：

```rust
# [derive(Clone, Debug, DeriveEntityModel, Deserialize, Serialize)]
# [sea_orm(table_name = "area", schema_name = "genshin_map")]
pub struct Model {
    pub version: i64,                       // 樂觀鎖
    #[sea_orm(primary_key)] pub id: i64,
    pub del_flag: bool,                     // 軟刪除
    pub name: String,
    pub parent_id: i64,
    // ...其餘字段
}
impl_safe_operation! { /* 列出 update_time/del_flag 列 */ }
```

**第 2 層 · DTO** `packages/utils/src/models/`：定義 `AreaListRequest`、
`AreaVO`、`AreaListResponse`，並以 `CommonResponse<T>` 統一包裝。

**第 3 層 · 業務** `packages/functions/src/functions/api/area.rs`：

```rust
pub async fn do_list(auth: AuthInfo, payload: AreaListRequest)
    -> Result<CommonResponse<AreaListResponse>>
{
    let rows = area_model::Entity::find_safety()      // 自動過濾 del_flag
        .filter(area_model::Column::ParentId.eq(payload.parent_id))
        .all(&*DB_CONN).await?;
    // 映射爲 VO、裝進 CommonResponse
}
```

**第 4 層 · 路由** `packages/router/src/routes/api/area/list.rs` + `mod.rs`：

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

**第 5 層 · 測試** `tests/rust/tests/area/area_domain_test.rs`：起一個真實（或
testcontainers）Postgres，調用路由做 list/get/add/update/delete 冒煙，校驗軟刪除
與樂觀鎖自增。

## 檢查清單

- [ ] 實體經 `impl_safe_operation!` 獲得軟刪除 + 樂觀鎖
- [ ] DTO 在 `utils` 包，業務層不直接返回 sea-orm `Model`
- [ ] 路由統一用 `ExtractAuthInfo` + `CommonResponse`
- [ ] 冒煙測試覆蓋至少 list/get/add/update/delete 五個路徑
- [ ] commit subject 符合 gitmoji 規範
