# Genshin Map Cloud — Rust バックエンド

> 「空荧酒館・原神マップ」バックエンドの Rust による書き直し。Java の参照実装
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud))
> と機能を同期しています。

これは日本語ドキュメントセクションです。バックエンドは `axum`・`sea-orm`
（PostgreSQL）・`redis`・`minio`・`jsonwebtoken` + `bcrypt` で構成された
4 パッケージの Cargo workspace（`utils → database → functions → router`）です。
このセクションは現在エントリーページのみで、完全なドキュメントは
[English](../en/README.md) または [简体中文](../zhs/README.md) を参照してください。

---

## ドキュメント索引

### ガイド

| ガイド | 内容 |
| --- | --- |
| [Detailed README](../en/guides/README.md) | プロジェクト概要・技術スタック・クイックスタート |
| [Glossary](../en/guides/glossary.md) | 中英ドメイン用語集 |
| [Architecture](../en/guides/architecture.md) | 4 パッケージのレイヤリング・リクエストフロー・`SafeEntityTrait` パターン |
| [Building](../en/guides/building.md) | 前提条件・`just` コマンド・`.env`・ローカル docker-compose |
| [API Reference](../en/guides/api-reference.md) | ルーターが公開する API ドメイン（area/icon/item/marker/punctuate/score/system…） |
| [Commit Convention](../en/guides/commit-message-convention.md) | gitmoji コミット規約 |
| [Java Sync Roadmap](../en/guides/sync-with-java-roadmap.md) | Java 参照実装からの移植優先順位 |
| [Domain Sync Template](../en/guides/domain-sync-template.md) | Java ドメインを Rust へ移植する 5 層テンプレート |

### 設計

- [Punctuate Workflow](../en/designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](../en/designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](../en/designs/hidden-and-special-flags.md)

---

## 他の言語

[简体中文](../zhs/README.md) · [English](../en/README.md) · [繁體中文](../zht/README.md) ·
**日本語** · [한국어](../ko/README.md) · [Français](../fr/README.md) ·
[Español](../es/README.md) · [Русский](../ru/README.md) · [العربية](../ar/README.md) ·
[Deutsch](../de/README.md) · [Português](../pt/README.md)
