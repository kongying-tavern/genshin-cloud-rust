# Hidden and Special Flags — Data-Level Visibility for Crowd-Sourced Map Data

> Enum: `HiddenFlag` in `packages/utils/src/types/common.rs`. Bitmask: the
> `special_flag` column on `item` and `area`. Query logic:
> `functions/api/item.rs::do_get_list`. Soft-delete guard: `SafeEntityTrait` in
> `packages/utils/src/db_operations.rs`. Java counterpart: `HiddenFlag` enum and
> the custom MyBatis query `selectPageItemByCondition`.

## Why two separate visibility mechanisms

A normal backend has one visibility concept: "can this user see this row?" The
空荧酒馆 map needs three, because its data is crowd-sourced, regional, and
spoilery all at once:

1. **Soft delete** (`del_flag`) — *does this row exist at all?* A marker removed

by an editor is logically gone but kept in the table for audit history.

1. **`hidden_flag`** — *which audience tier is allowed to know this marker

exists?* Gates spoilers, insider data, and test-server data.

1. **`special_flag`** — *within the visible set, does this row match the

player's current UI filter?* A pure bitmask applied to the item/area browsing
UI.

They are orthogonal. A soft-deleted row is hidden from everyone regardless of
its `hidden_flag` or `special_flag`. A `hidden_flag = Spy` row is hidden from
the public regardless of its `special_flag`. This document explains the last
two; soft delete is covered by the `SafeEntityTrait` pattern in
[the architecture guide](../guides/architecture.md).

## `hidden_flag` — the audience tier

`HiddenFlag` is a `DeriveActiveEnum` stored as an `i32` on every content entity
(`marker`, `item`, `area`, `route`, `tag`). The values are fixed by the Java
enum and the wire contract; do not renumber them.

| Variant | Value | Chinese | Audience |
| --- | --- | --- | --- |
| `Visible`  | 0 | 可见 | Everyone. The public dataset. |
| `Hidden`   | 1 | 隐藏 | Insiders only — internal contributors, not yet released. |
| `Spy`      | 2 | 内鬼 / 测试服 | Test-server / datamined data. Must not leak to the public map. |
| `Suprise`  | 3 | 彩蛋 | Easter-egg content (note the Java spelling `Suprise`, preserved for parity). |

```rust
pub enum HiddenFlag {
    #[sea_orm(num_value = 0)] Visible  = 0,
    #[sea_orm(num_value = 1)] Hidden   = 1,
    #[sea_orm(num_value = 2)] Spy      = 2,
    #[sea_orm(num_value = 3)] Suprise  = 3,
}
```

### How the gate works

Visibility is decided on the client side, driven by a bitmask the client sends
in a `userDataLevel` header (the Java name; the Rust router reads it on the
`/api/*` paths). The header is a bitmask over the `HiddenFlag` numeric values:

- `userDataLevel = 1` (bit 0 set) → may see `Visible` only — the default for an

ordinary player.

- Higher bits grant the corresponding tiers. Insider accounts get the bit for

`Hidden`; internal/test accounts get the bits for `Spy` and `Suprise`.

The server's job is therefore not to *decide* visibility per user, but to
**partition the dataset by flag** so the client can pick the partitions its
`userDataLevel` allows. This is exactly why the BinaryMD5 export pipeline (see
[BinaryMD5 archive export](./binarymd5-archive-export.md)) groups every entity
by `hidden_flag` before compressing: each flag becomes its own blob with its own
MD5, and the client simply requests only the blobs its level permits.

The same grouping appears in the synchronous read paths. In
`functions/api/area.rs::do_list`, for example, the client passes the
`hidden_flag` it wants and the filter is applied directly:

```rust
if let Some(hidden_flag) = payload.hidden_flag {
    query = query.filter(area_model::Column::HiddenFlag.eq(hidden_flag));
}
```

### Why this is not a permission check

`hidden_flag` is a *data-level* filter, not an authorization boundary. The
server hands the client the blob for `flag = Visible` and trusts the client not
to ask for `flag = Spy` if its `userDataLevel` does not allow it. This is
deliberate:

- The map is a public read-mostly tool; there is no privileged write path to

protect here (writes go through the punctuate audit, see
[Punctuate workflow](./punctuate-workflow.md)).

- The insider/test data is "soft confidential" — embarrassing if leaked, not a

security incident. The cost of a server-side enforcement layer per request is
not worth it for that threat model.

- The spoiler/region-lock UX (a player who has not unlocked Inazuma should not

see Inazuma-specific surprises) is naturally client-driven: the client knows
the player's progress, the server does not.

The hard boundary — preventing vandalism — is handled upstream by the punctuate
audit queue, not by `hidden_flag`.

## `special_flag` — the bitmask filter

`special_flag` is an `Option<i32>` on `item` (nullable) and a non-nullable `i32` on `area`. Its bits are
defined by front-end convention (e.g. "this item only appears at night", "this
area requires a quest to access") and the server treats them as opaque bits.

It is *not* an audience gate. Two `Visible` items that differ only in
`special_flag` are both visible to everyone; `special_flag` just lets the
client's filter UI show or hide them.

### The query contract

The bitmask filter in `functions/api/item.rs::do_get_list` is the direct port of
Java's `selectPageItemByCondition`. The contract has two branches, and both
matter:

```rust
if let Some(sf) = payload.special_flag {
    let sf = sf as i32;
    if sf == 0 {
        // "items with no special mark at all"
        query = query.filter(item_model::Column::SpecialFlag.eq(0));
    } else {
        // "items with any of these bits set"
        query = query.filter(
            Expr::col(item_model::Column::SpecialFlag).bit_and(sf).ne(0)
        );
    }
}
```

| Client sends | Meaning | SQL shape |
| --- | --- | --- |
| `special_flag = 0` | "Give me only items with no special mark." | `WHERE special_flag = 0` |
| `special_flag > 0` | "Give me items matching any of these bits." | `WHERE (special_flag & ?) != 0` |
| `special_flag` omitted | No filter; return all items regardless of bits. | (no clause) |

The asymmetry — `0` means *equality*, not *match-nothing* — is the part that
trips people up. It exists because `special_flag = 0` has a real semantic
meaning in the UI: "the plain, un-marked items." Treating `0` as `(x & 0) != 0`
would match nothing (since `x & 0` is always `0`), which is useless; instead
the contract turns `0` into an exact-match query for the un-marked set. This
matches the Java MyBatis query byte-for-byte.

The `area` table carries the same `special_flag` column and follows the same
contract.

## Interaction with soft delete

`hidden_flag` and `special_flag` only ever narrow the visible set *within live
rows*. They never resurrect a deleted row.

Every read path in the project goes through `find_safety()` (or
`find_safety_by_id(id)`), generated by the `impl_safe_operation!` macro:

```rust
fn find_safety() -> Select<Self> {
    Self::find().filter(/* del_flag = false */)
}
```

So the effective visibility of any row is the conjunction of three independent
predicates:

```text
visible(row) = (del_flag == false)                // hard floor: row exists
            ∧ (hidden_flag ⊆ userDataLevel)        // audience tier: client decides
            ∧ (special_flag matches filter)        // UI bitmask: client decides
```

A soft-deleted marker is invisible to the public, to insiders, and to test
accounts alike — `find_safety()` excludes it before the other two clauses even
get a chance to run. Editors can still see deleted rows by going through the
history/audit tables, not the live `find_safety()` path. This is why the
punctuate `Deleted` path (see [Punctuate workflow](./punctuate-workflow.md))
uses `delete_safety` (which sets `del_flag = true`) rather than a hard delete:
the row drops out of every live view immediately, but stays recoverable for
audit.

## Common mistakes to avoid

- **Treating `hidden_flag` like a boolean "hidden/not-hidden".** Only `Visible`

(0) is public; `Hidden`, `Spy`, and `Suprise` are three *different*
non-public audiences, and a `userDataLevel` bitmask grants each independently.

- **Rewriting the `special_flag = 0` branch as `(x & 0) != 0`.** That matches

nothing and silently empties the UI's "plain items" view.

- **Adding a raw `Entity::find()` to read live data.** It bypasses the

`del_flag` floor and will return rows that every other path agrees are gone.
Use `find_safety()` always.

- **Renumbering `HiddenFlag` "for clarity".** The numeric values are the wire

and DB contract with the Java implementation and every shipped client. The
misspelling of `Suprise` is likewise intentional parity with the Java enum.
