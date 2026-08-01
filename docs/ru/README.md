# Genshin Map Cloud — бэкенд на Rust

> Переписывание бэкенда «空荧酒馆 Genshin Map» на Rust с синхронизацией
> функций с эталонной реализацией на Java
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud)).

Это раздел документации на русском языке. Бэкенд представляет собой Cargo
workspace из четырёх пакетов (`utils → database → functions → router`) на
`axum`, `sea-orm` (PostgreSQL), `redis`, `minio`, с `jsonwebtoken` + `bcrypt`
для аутентификации. Этот раздел пока является только входной страницей;
полная документация доступна на [English](../en/README.md) или
[简体中文](../zhs/README.md).

---

## Индекс документации

### Руководства

| Руководство | Содержание |
| --- | --- |
| [Detailed README](../en/guides/README.md) | Обзор проекта, стек, быстрый старт |
| [Glossary](../en/guides/glossary.md) | Китайско-английская терминология |
| [Architecture](../en/guides/architecture.md) | Слои четырёх пакетов, поток запросов, паттерн `SafeEntityTrait` |
| [Building](../en/guides/building.md) | Требования, команды `just`, файл `.env`, локальный docker-compose |
| [API Reference](../en/guides/api-reference.md) | Домены API роутера (area/icon/item/marker/punctuate/score/system…) |
| [Commit Convention](../en/guides/commit-message-convention.md) | Соглашение о коммитах gitmoji |
| [Java Sync Roadmap](../en/guides/sync-with-java-roadmap.md) | Приоритеты переноса из Java-реализации |
| [Domain Sync Template](../en/guides/domain-sync-template.md) | Шаблон из пяти слоёв для переноса Java-домена в Rust |

### Дизайн

- [Punctuate Workflow](../en/designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](../en/designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](../en/designs/hidden-and-special-flags.md)

---

## Другие языки

[简体中文](../zhs/README.md) · [English](../en/README.md) · [繁體中文](../zht/README.md) ·
[日本語](../ja/README.md) · [한국어](../ko/README.md) · [Français](../fr/README.md) ·
[Español](../es/README.md) · **Русский** · [العربية](../ar/README.md) ·
[Deutsch](../de/README.md) · [Português](../pt/README.md)
