# Design Notes

Design notes capture the **why** behind non-obvious decisions in the Genshin Map
Cloud Rust backend. They are not step-by-step guides (see
[the guides](../guides/)) — they are the background a contributor needs to
understand *why* the code is shaped the way it is, especially where the Rust
port preserves a non-obvious decision from the Java reference implementation.

The 空荧酒馆 Genshin Map is a crowd-sourced interactive map. Three of its
properties drive almost every design decision in this directory:

1. **The data is contributed by players**, not authored by a paid team — so

every write flows through an audit gate before it becomes live data.

1. **The dataset is large and read-mostly** — the client cold-starts by

downloading the whole map, so the read path is optimized for bulk transfer.

1. **Some data is spoilery or test-only** — visibility is partitioned by

audience tier and by UI filter, not by a single permission check.

## Index

| Document | What it explains |
| --- | --- |
| [BinaryMD5 Archive Export](./binarymd5-archive-export.md) | Why the map cold-starts from GZIP-compressed JSON blobs keyed by MD5, how markers are paged by `id / 3000` while items and linkages use single-blob variants, and how the two-level cache (in-process `moka` + Redis) keeps regeneration off the request path. |
| [Hidden and Special Flags](./hidden-and-special-flags.md) | Why the project has three orthogonal visibility mechanisms — soft delete (`del_flag`), audience tier (`hidden_flag`), and UI bitmask (`special_flag`) — and how they compose. |

## Conventions

- Each note names the Rust file(s) and the Java counterpart it ports, so you can

follow the parity in both directions.

- Cross-references between notes use relative links; the guides are under

[`../guides/`](../guides/).

- When a Rust file knowingly diverges from Java (e.g. the BinaryMD5 cache layers

a TTL-bounded moka + Redis design under Java's never-refreshing Caffeine), the
note says so explicitly rather than papering over it.
