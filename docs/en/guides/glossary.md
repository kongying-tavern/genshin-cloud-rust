# Domain Glossary

> A Chinese ↔ English ↔ code-identifier dictionary for the 空荧酒馆 Genshin Map.
> Use this to move between the user-facing Chinese terminology (what the
> contributors and players call things), the English docs (this tree), and the
> actual Rust/Java identifiers in the codebase.
> The project is a Rust rewrite of
> [`genshin-map-cloud`](https://github.com/kongying-tavern/genshin-map-cloud).
> Identifier names are kept in lockstep across both codebases; where a name is
> surprising (e.g. the misspelling of `Suprise`), it is preserved for parity
> and called out here.

## Core content entities

These are the tables that hold the map's actual data. All live under the
`genshin_map` Postgres schema and follow the `SafeEntityTrait` pattern
(`version` optimistic lock + `del_flag` soft delete) — see
[the architecture guide](./architecture.md).

| Chinese | English | Code identifier | Notes |
| --- | --- | --- | --- |
| 点位 | marker (POI) | `marker` table, `MarkerVO` | A point of interest on the map: coordinates + icon + items + content. The unit of crowd-sourced contribution. |
| 打点 | punctuate (crowd-sourced submission) | `marker_punctuate` table | A *staged* contribution that has not yet been audited (the audit workflow is deprecated; the staging table remains part of the schema). |
| 物品 | item (collectible) | `item` table, `ItemVO` | A collectible/resource type (e.g. a particular kind of ore, a regional specialty, a chest tier). Attached to markers via `marker_item_link`. |
| 图标 | icon (marker visual) | `icon` table | The picture drawn on top of a marker. Item → icon → marker is the rendering chain. |
| 地区 | area (game region) | `area` table, `AreaVO` | A game region: 蒙德 (Mondstadt), 璃月 (Liyue), 稻妻 (Inazuma), 须弥 (Sumeru), 枫丹 (Fontaine), 纳塔 (Natlan), 至冬 (Snezhnaya), etc. Hierarchical via `parent_id`. |
| 路线 | route (farming path) | `route` table | A predefined farming path drawn through multiple markers. Distinct from `marker_linkage`, which is a relationship edge. |
| 神瞳 | Oculus (collectible type) | — (represented as an icon style) | Anemoculus (风神瞳), Geoculus (岩神瞳), Electroculus (雷神瞳), Dendroculus (草神瞳), Hydroculus (水神瞳), Pyroculus (火神瞳), Cryoculus (冰神瞳). Not a separate table; modeled as items with a specific icon/style. |
| 宝箱 | chest | — (item + icon) | Likewise modeled as an item/icon combination rather than its own table. |

## Tagging and categorization

| Chinese | English | Code identifier | Notes |
| --- | --- | --- | --- |
| 标签 | tag | `tag` table | A label applied to icons/markers for grouping. |
| 图标分类 / 标签分类 | icon_type / tag_type (category) | `icon_type`, `tag_type` tables | Category buckets for icons and tags. The two exist because icons and tags are categorized independently. |
| 点位关联 | marker_linkage (path/edge between markers) | `marker_linkage` table, `MarkerLinkageLinkAction` | A directed or undirected edge between two markers (`from_id` ↔ `to_id`), classified by a `link_action` (trigger, related, path, equivalent, …). Used to render "follow this route" lines and trigger groups. |

The many-to-many join tables follow a consistent `*_link` suffix:

| Join | Connects |
| --- | --- |
| `marker_item_link` | marker ↔ item (which collectibles this POI gives) |
| `item_type_link` | item ↔ item_type (category membership) |
| `icon_type_link` | icon ↔ icon_type |
| `tag_type_link` | tag ↔ tag_type |

## Visibility and audit

| Chinese | English | Code identifier | Notes |
| --- | --- | --- | --- |
| 权限屏蔽标记 | hidden_flag (audience tier) | `HiddenFlag` enum | Data-level audience gate: `Visible` / `Hidden` / `Beta` / `Suprise`. See [Hidden and special flags](../designs/hidden-and-special-flags.md). |
| 特殊标记 | special_flag (UI bitmask) | `special_flag: Option<i32>` column | A bitmask filter applied by the client's item/area browsing UI. |
| 审核流程 | punctuate audit workflow (deprecated) | `MarkerPunctuateStatus` enum | The `Pending → Reviewing → Rejected` state machine no longer ships in the Rust port; the `marker_punctuate` staging table stays for schema parity. |
| 暂存 | stage (Pending) | `MarkerPunctuateStatus::Pending` (Java `STAGE`) | A submission saved by the contributor but not yet handed to editors. |
| 审核中 | reviewing (committed) | `MarkerPunctuateStatus::Reviewing` (Java `COMMIT`) | A submission in the editor queue. |
| 不通过 | rejected | `MarkerPunctuateStatus::Rejected` (Java `REJECT`) | An editor turned the submission down; the contributor can revise and re-commit. |
| 审核备注 | audit remark | `audit_remark` column | Free-text reason stored on rejection. |
| 逻辑删除 | soft delete | `del_flag: bool` column | Orthogonal to the flags above; `find_safety()` excludes these rows for every audience. |

## Export and scoring

| Chinese | English | Code identifier | Notes |
| --- | --- | --- | --- |
| BinaryMD5 (压缩归档) | BinaryMD5 (compressed archive export) | `*_doc` endpoints, `BinaryMd5Vo`, `serialize_compress_md5` | GZIP-compressed JSON blobs keyed by MD5 of the compressed bytes, served for client cold-start. See [BinaryMD5 archive export](../designs/binarymd5-archive-export.md). |
| 评分统计 / 贡献度 | score_stat / contributor scoring | `score_stat` table, `do_generate_score` | Aggregated contribution counts per user, derived from the audit history. |
| 历史记录 | history (audit log) | `history` table, `HistoryEditType` / `HistoryOperationType` | Records every promoted punctuate and every editor write, classified by operation (area/icon/item/position) and edit type (added/modified/deleted). |

## Organization and meta terms

| Chinese | English | Notes |
| --- | --- | --- |
| 空荧酒馆 | Kongying Tavern | The community/organization that maintains the map. The GitHub org is `kongying-tavern`. Sometimes abbreviated 空荧. |
| 原神地图 | Genshin Map | The product itself. The Rust backend in this repo is the server half; the Java repo is the reference implementation. |
| 提瓦特 | Teyvat | The game world. "All of Teyvat" is the dataset the `*_doc` exports aim to cover. |
| 测试服 | Beta (test-server data) | `HiddenFlag::Beta` (legacy name `Spy`). Test-server content that must not leak to the public map. |
| 彩蛋 | easter egg | `HiddenFlag::Suprise` (misspelled to match the Java enum). |

## Reading the table

A few conventions that hold across the codebase:

- **Table names** (`table_name = "..."`) are singular and match the Java MyBatis

mapping one-to-one. Do not rename them.

- **VO suffix** (`MarkerVO`, `ItemVO`, `AreaVO`) marks the wire DTO returned by

`do_get_*` functions; the `Model` in `packages/database/src/models/...` is the
sea-orm row representation. They are deliberately separate types so the wire
shape can evolve without coupling to the schema.

- **`do_*` functions** in `functions/api/<domain>.rs` are the business-logic

entry points; every axum handler in `routes/api/<domain>/` is a thin wrapper
that parses the request and delegates to one.

- **Java parity comments** (`/// 对应 Java ...`) appear at the top of most

functions and are the authoritative cross-reference when a behavior looks
surprising.
