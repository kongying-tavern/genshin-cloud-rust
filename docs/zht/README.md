# Genshin Map Cloud — Rust 後端

> 「空熒酒館·原神地圖」後端的 Rust 重寫，與 Java 參考實作
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud))
> 功能對齊。

這是繁體中文文件區。後端是以 `axum`、`sea-orm`（PostgreSQL）、`redis`、
`minio`、`jsonwebtoken` + `bcrypt` 組成的四套件 Cargo workspace
（`utils → database → functions → router`）。本區目前為入口頁，
完整文件請見 [English](../en/README.md) 或 [简体中文](../zhs/README.md)。

---

## 文件索引

### 指南（Guides）

| 指南 | 內容 |
| --- | --- |
| [Detailed README](../en/guides/README.md) | 專案總覽、技術棧、快速開始 |
| [Glossary](../en/guides/glossary.md) | 中英領域術語對照 |
| [Architecture](../en/guides/architecture.md) | 四套件分層、請求流、`SafeEntityTrait` 模式 |
| [Building](../en/guides/building.md) | 前置條件、`just` 指令、`.env`、本機 docker-compose |
| [API Reference](../en/guides/api-reference.md) | 路由暴露的 API 域（area/icon/item/marker/punctuate/score/system…） |
| [Commit Convention](../en/guides/commit-message-convention.md) | gitmoji 提交規範 |
| [Java Sync Roadmap](../en/guides/sync-with-java-roadmap.md) | 從 Java 參考實作移植的優先順序 |
| [Domain Sync Template](../en/guides/domain-sync-template.md) | 單一 Java 域移植到 Rust 的五層模板 |

### 設計（Designs）

- [Punctuate Workflow](../en/designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](../en/designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](../en/designs/hidden-and-special-flags.md)

---

## 其他語言

[简体中文](../zhs/README.md) · [English](../en/README.md) · **繁體中文** ·
[日本語](../ja/README.md) · [한국어](../ko/README.md) · [Français](../fr/README.md) ·
[Español](../es/README.md) · [Русский](../ru/README.md) · [العربية](../ar/README.md) ·
[Deutsch](../de/README.md) · [Português](../pt/README.md)
