# Genshin Map Cloud — Rust Backend

> The Rust rewrite of the "空荧酒馆 Genshin Map" backend, feature-synced with
> the Java reference implementation
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud)).

This is the English documentation section. The backend is organized as a
four-package Cargo workspace (`utils → database → functions → router`) built on
`axum`, `sea-orm` (PostgreSQL via `sqlx`), `redis`, and `minio`, with
`jsonwebtoken` + `bcrypt` for authentication and `tokio` + `tracing` for the
runtime and observability stack.

The goal of this section is to give a new contributor everything needed to
build, run, and extend the backend — from the overall architecture down to the
template for porting a single Java domain into Rust.

---

## Documentation Index

### Guides

Hands-on, task-oriented documents. Read them in any order; the suggested entry
path is **architecture → building → api-reference**, then the sync/commit
guides when you start contributing.

| Guide | What it covers |
| --- | --- |
| [Detailed README](./guides/README.md) | Project overview, tech stack, quick start |
| [Glossary](./guides/glossary.md) | Chinese-English domain terminology |
| [Architecture](./guides/architecture.md) | The four-package layering, request flow from axum to PostgreSQL, the `SafeEntityTrait` pattern, and Redis/MinIO integration points. |
| [Building](./guides/building.md) | Prerequisites, `just init` / `just build` / `just dev`, the `.env` file, the local docker-compose stack, and the CI workflows. |
| [API Reference](./guides/api-reference.md) | The API domains the router exposes (area, icon, item, marker, notice, punctuate, score, system, ...), grouped by purpose. |
| [Commit Convention](./guides/commit-message-convention.md) | The `celestia-devtools` gitmoji convention enforced by the commit-msg hook, with the common gitmoji cheatsheet and skip overrides. |
| [Java Sync Roadmap](./guides/sync-with-java-roadmap.md) | The priority order for porting features from `java-genshin-map-cloud`, with the key entity/feature and complexity estimate for each step. |
| [Domain Sync Template](./guides/domain-sync-template.md) | The five-layer pattern for porting one Java domain to Rust, with a concrete `area` mini-example. |

### Designs

Design notes capture the "why" behind non-obvious decisions. This section is
seeded alongside the guides above; deeper ADR-style documents will be added as
the port progresses.

- [Punctuate Workflow](./designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](./designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](./designs/hidden-and-special-flags.md)

---

## Other languages

The documentation tree is multilingual and rendered with
[`lagrange`](https://github.com/celestia-island/lagrange). English is the
default language; the other translations are scaffolded and link back here
until they are complete.

[简体中文](../zhs/README.md) · **English** · [繁體中文](../zht/README.md) ·
[日本語](../ja/README.md) · [한국어](../ko/README.md) · [Français](../fr/README.md) ·
[Español](../es/README.md) · [Русский](../ru/README.md) · [العربية](../ar/README.md) ·
[Deutsch](../de/README.md) · [Português](../pt/README.md)
