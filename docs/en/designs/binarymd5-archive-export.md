# BinaryMD5 Archive Export — Compressed Bulk Read for Client Cold-Start

> Domains: `marker_doc`, `item_doc`, `marker_link_doc`, `icon_doc`,
> `tag_doc`. Pipeline core: `functions/api/binary_doc.rs`. Java counterpart:
> `CompressUtils` + `DigestUtils`, `*DaoImpl.refresh*BinaryList`, and the
> `neverRefreshCacheManager` Caffeine cache; served by `*DocController`.

## Why this pipeline exists

The map front-end does not fetch POIs one at a time. When a player opens the
map, the client needs **the entire dataset** for the regions they care about:
every Anemoculus (风神瞳), every chest (宝箱), every specialty node, every item
definition, every inter-marker linkage. For the 空荧酒馆 map that is tens of
thousands of records across the whole of Teyvat.

Serving this over the normal paginated JSON endpoints is hopeless:

- Round-trip per page adds up to seconds of latency on a cold map.
- Per-record JSON serialization is wasteful when 99% of the payload is the same

fields repeated.

- Most cold-starts re-download data the client already has, because the server

cannot tell which records changed.

The BinaryMD5 pipeline solves all three by turning the dataset into a small set
of **GZIP-compressed JSON blobs**, each addressed by the MD5 of its compressed
bytes. The client keeps the last MD5 it saw per blob; on cold-start it asks only
for the MD5 list, diffs against its local cache, and downloads just the blobs
that changed. Everything ships as raw `application/octet-stream`, not JSON over
HTTP, so the wire payload is the smallest useful representation.

## The pipeline

`binary_doc.rs::serialize_compress_md5<T: Serialize>` is the one function every
`*_doc` domain routes through. It mirrors the Java `CompressUtils` +
`DigestUtils` sequence exactly:

```text
  Vec<T>            serde_json::to_vec             GzEncoder              md5::compute
 ──────────  ───────────────────────────►  ────────────────────►  ──────────────────►
  entities   ──►  JSON UTF-8 bytes          GZIP compress          lowercase hex MD5
                                                                       (32 chars)

                                                  ▲
                              MD5 is computed over the COMPRESSED bytes, not the JSON.
```

Order matters: MD5 over compressed bytes means the client can verify a download
by hashing the exact bytes it received, without ever needing to decompress
first. The MD5 is simultaneously the cache key, the ETag, and the content
address.

## Grouping and paging

The blobs are not "one giant file." Each `*_doc` domain groups the dataset
before compressing, so a small change only invalidates the blob it belongs to.

The grouping rules live in the per-domain files and mirror the Java
`refresh*BinaryList` methods.

### Markers (`marker_doc.rs`)

Markers are the largest dataset, so they get the most aggressive paging:

```text
  all markers (find_safety)
        │
        ▼
  group by hidden_flag            (BTreeMap<i32, …> → ascending flag order)
        │
        ├── flag == Visible (0)  ──► split further: page_index = marker.id / 3000
        │                              each page → its own MD5
        │
        └── flag  ∈ {Hidden, Beta, Suprise}
                                  ──► single page (index 0) → one MD5 for the whole flag
```

Two non-obvious choices:

1. **Why page by `id / 3000`, not by a sliding window?** Stable membership. As

long as marker ids are allocated monotonically, a marker stays on the same
page forever — a new marker does not shift existing markers into a different
page the way offset-based paging would. A single insertion invalidates
exactly one page.

1. **Why is `Visible` the only flag that gets paged?** It is by far the biggest

group (every public marker). The insider-only, test-server, and easter-egg
groups (see [Hidden and special flags](./hidden-and-special-flags.md)) are
small enough that one blob each is fine. See `MARKER_PAGE_SIZE` in
`marker_doc.rs`.

`MARKER_PAGE_SIZE` is 3000. That number is a deliberate trade-off:

- **Fewer, larger pages** would mean fewer requests on a fully warm cache, but a

single edit would re-download a huge blob, and the per-page GZIP blob would
start to dominate memory.

- **More, smaller pages** would make incremental edits cheaper, but a cold map

would issue many round-trips.

3000 keeps a typical cold-start in the low double digits of page requests while
keeping each page's compressed size modest (a few hundred KB).

### Items (`item_doc.rs`)

Items are grouped by `hidden_flag` only — each flag is a single page (index 0),
no `id / 3000` split. Items are far less numerous than markers, so the extra
paging is not worth it.

### Marker linkages (`marker_link_doc.rs`)

Linkages (the edges between markers — see
[the glossary](../guides/glossary.md) under 点位关联) use a **single-blob
variant**: the entire dataset is one GZIP blob, one MD5, no paging and no
per-flag grouping. Two views are exposed:

- **`list`** — the flat `Vec<MarkerLinkage>` array.
- **`graph`** — an adjacency map `HashMap<marker_id, Vec<marker_id>>`

precomputed server-side so the client does not have to rebuild it.

Icons (`icon_doc.rs`) and tags (`tag_doc.rs`) use the same single-blob shape:
the whole set is one GZIP blob served by `all_bin_md5` / `all_bin`, with no
paging and no grouping.

## Two endpoints per domain

Every `*_doc` domain exposes the same pair of endpoints: the first is what the
client polls; the second is what it downloads only when an MD5 changed. The
linkage domain exposes the pair twice — once for the `list` view and once for
the `graph` view.

| Endpoint | Returns | Purpose |
| --- | --- | --- |
| `list_page_bin_md5` (markers/items), `all_bin_md5` (icons/tags), or `all_list_bin_md5` / `all_graph_bin_md5` (linkages) | `Vec<BinaryMd5Vo>` of `{ md5, time }` | Cheap poll. Client diffs `md5` against its cache. |
| `list_page_bin/{md5}` (markers/items), `all_bin` (icons/tags), or `all_list_bin` / `all_graph_bin` (linkages) | raw `application/octet-stream` bytes | The compressed blob. `md5` is the content address. |

`BinaryMd5Vo` is defined in `binary_doc.rs`:

```rust
# [serde(rename_all = "camelCase")]
pub struct BinaryMd5Vo { pub md5: String, pub time: i64 }
```

`time` is the generation timestamp (epoch millis) shared by every entry in a
single `list_page_bin_md5` response. It is metadata for the client's UI ("this
snapshot is from when"), not part of the cache key.

The `{md5}` in the fetch URL is doing the job of an ETag, but baked into the
path so the endpoint is trivially cacheable by any HTTP layer (CDN, Nginx,
browser) without conditional-request negotiation.

## Caching: what Rust does today vs. Java

Rust used to regenerate every page on each request; it now caches at two
levels, both implemented in `binary_doc.rs`:

- **In-process moka cache.** `get_or_compute(key, compute)` stores each

`CachedPage { md5, time, bytes }` in a moka cache (10,000-entry capacity,
3600s TTL) keyed by an explicit domain + group/page string (`item:0`,
`marker:0:123`, `link:graph`). The cached `time` stays stable while an entry
is alive, so the timestamps in the md5 list do not churn between requests.

- **Redis second-level cache.** A result-level cache under `binmd5:result:*`

shares the computed page sets across replicas, so a warm replica serves the
pages without re-scanning the database. Invalidation bumps a versioned epoch
(`binmd5:epoch`), which makes every replica drop its stale copy at once; old
keys simply age out of the TTL window.

Invalidation is wired to the admin surface: `POST /app/trigger/update` and the
`DELETE /api/cache/{item,marker,marker_link}` endpoints flush every cached
page (in-process + Redis across replicas) and broadcast a purge event over the
WebSocket layer. The remaining `/api/cache/*` sub-routes are honest no-ops
until their domains grow a cache layer.

- **Java** precomputes every page into Caffeine's `neverRefreshCacheManager`

(no TTL, no refresh; entries are invalidated and re-added only when a
`refresh*BinaryList` job runs after a write). Both `list_page_bin_md5` and
`list_page_bin/{md5}` are then O(1) hashmap reads.

- **Rust today** computes each page lazily on the first request of a TTL

window and serves it from moka (and Redis) afterwards, instead of Java's
write-time precomputation. That trade keeps cold paths correct — the bytes
are deterministic, so the MD5s always match — while bounding staleness by
the TTL and the explicit flush endpoints.

The `serialize_compress_md5 → (bytes, md5)` split is the seam the cache plugs
into: the lookup wraps it directly, and nothing downstream needed to change.

## Why MD5-over-compressed and not MD5-over-JSON

Two reasons, both load-bearing:

1. **End-to-end verifiability.** The client hashes the bytes it received off the

wire; if the hash matches the MD5 in the URL, the download is intact. If the
MD5 were computed over the JSON, the client would have to decompress before
it could verify integrity.

1. **Cache addressability.** The compressed bytes are the actual cached value,

so keying by their MD5 means "the bytes for this MD5" is a literal identity,
not a claim that needs a separate verification step. A CDN can cache the
response purely on the URL.

The cost is that adding a new compression level or swapping GZIP for something
else invalidates every MD5 at once. That is acceptable — it happens roughly
never — and is the same trade-off Java made.
