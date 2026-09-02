# Domain Sync Template

A reusable checklist for porting a single domain from the Java backend to
Rust. The `area` domain is used as the concrete example — follow this pattern
for every new domain (icon, item, tag, notice, ...). It is the template the
`area` + `marker` reference samples (step 1 of the
[Java Sync Roadmap](./sync-with-java-roadmap.md)) established.

## The five layers

Every domain touches exactly five locations, one per layer. Port them in this
order so each layer's dependencies already exist when you reach it.

### 1. sea-orm entity — `packages/database/src/models/<domain>/<domain>.rs`

Define the `Model`, `ActiveModel` (via `DeriveEntityModel`), `Column`,
`Relation`, and the `impl_safe_operation!` invocation. The entity **must**
include `version` (optimistic lock), `del_flag` (soft delete), and — for
content entities — `hidden_flag` (data-level filtering). Register the module
in `packages/database/src/models/<domain>/mod.rs` and the parent `mod.rs`.

```rust
# [derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
# [sea_orm(table_name = "area", schema_name = "genshin_map")]
pub struct Model {
    pub version: i64,
    #[sea_orm(primary_key)]
    pub id: i64,
    pub create_time: DateTime,
    pub update_time: Option<DateTime>,
    pub creator_id: Option<i64>,
    pub updater_id: Option<i64>,
    pub del_flag: bool,
    // ... domain fields ...
    pub hidden_flag: HiddenFlag,
    pub special_flag: i32,
}

impl_safe_operation! {
    active_model_ty: ActiveModel,
    updated_at_column_name: update_time,
    updated_at_column_init_expr: chrono::Utc::now().naive_utc(),
    del_flag_column: Column::DelFlag
}
```

### 2. DTO / VO types — `packages/utils/src/models/<domain>.rs`

Request and response shapes, serialized as `camelCase` to match the Java API
contract (`#[serde(rename_all = "camelCase")]`). Add the module to
`packages/utils/src/models/mod.rs`. Use `#[serde(flatten)]` to compose an
update request around the base request without repeating fields.

```rust
# [derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
# [serde(rename_all = "camelCase")]
pub struct AreaListRequest {
    pub parent_id: Option<i64>,
    pub is_traverse: Option<bool>,
}

# [derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
# [serde(rename_all = "camelCase")]
pub struct AreaUpdateRequest {
    pub id: i64,
    pub version: i64,
    #[serde(flatten)]
    pub area: AreaRequest,
}
```

### 3. Business logic — `packages/functions/src/functions/api/<domain>.rs`

Pure async functions (`do_list`, `do_get`, `do_add`, `do_update`,
`do_delete`) that orchestrate entity reads/writes. **No HTTP types here** —
they take DTOs and `AuthInfo`, return `CommonResponse<T>`, and reach the
database through `&DB_CONN.wait().pg_conn`. Always use the `*_safety*`
methods; never raw `Entity::find` / `update` / `delete`.

```rust
pub async fn do_add(auth: AuthInfo, payload: AreaAddRequest)
    -> Result<CommonResponse<AreaAddResponse>>
{
    let active = area_model::ActiveModel {
        version: Set(0),
        creator_id: Set(Some(auth.info.id)),
        del_flag: Set(false),
        // ... map payload fields ...
        ..Default::default()
    };
    let res = active.insert(&DB_CONN.wait().pg_conn).await?;
    Ok(CommonResponse::new(Ok(AreaAddResponse { id: res.id })))
}

pub async fn do_delete(_auth: AuthInfo, area_id: i64)
    -> Result<CommonResponse<EmptyResponse>>
{
    let item = area_model::Entity::find_safety_by_id(area_id)
        .one(&DB_CONN.wait().pg_conn).await?
        .ok_or(anyhow!("Area not found"))?;
    let mut am: area_model::ActiveModel = item.into();
    am.del_flag = Set(true);
    area_model::Entity::delete_safety(am)
        .exec(&DB_CONN.wait().pg_conn).await?;
    Ok(CommonResponse::new(Ok(EmptyResponse {})))
}
```

### 4. axum routes — `packages/router/src/routes/api/<domain>/`

One file per verb (`add.rs`, `get.rs`, `list.rs`, `update.rs`, `delete.rs`,
plus `mod.rs` that wires them into a `Router`). Each handler extracts
`ExtractAuthInfo` and the request body, delegates to the matching `do_*`
function, and maps the `Result` into a status code. Register the domain in
the parent `packages/router/src/routes/api/mod.rs` via
`.nest("/<domain>", <domain>::router().await?)`.

```rust
// packages/router/src/routes/api/area/mod.rs
pub async fn router() -> Result<Router> {
    let ret = Router::new()
        .route("/get/list", post(list::list))
        .route("/get/{area_id}", post(get::get))
        .route("/add", put(add::add))
        .route("/update", post(update::update))
        .route("/{area_id}", delete(delete::delete));
    Ok(ret)
}
```

### 5. Smoke tests — `tests/rust/tests/<domain>/`

At minimum, assert the entity table name matches the Java side and that the
`hidden_flag` / `version` columns exist (see
`tests/rust/tests/area/area_domain_test.rs`). These are compile-time + value
assertions and need no database connection. Add DB-backed integration tests
under `#[ignore]` once the docker-compose harness is wired up.

```rust
# [test]
fn area_entity_table_name_matches_java() {
    assert_eq!(Entity.table_name(), "area");
}
```

## Checklist

- [ ] Entity matches the Java table name + schema (`genshin_map`)
- [ ] `version`, `del_flag`, `hidden_flag` columns present
- [ ] `impl_safe_operation!` invoked (generates `find_safety` /

`update_safety` / `delete_safety`)

- [ ] DTO/VO fields are `camelCase` (Java parity)
- [ ] `do_list` supports `hidden_flag` filtering where the Java side does
- [ ] Route handlers registered in the parent `routes/api/mod.rs`
- [ ] Smoke test asserts table name + key columns
