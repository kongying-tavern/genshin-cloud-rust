# Genshin Map Cloud — Backend Rust

> La reescritura en Rust del backend « 空荧酒馆 Genshin Map », sincronizada con
> la implementación de referencia en Java
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud)).

Esta es la sección de documentación en español. El backend es un workspace de
Cargo con cuatro paquetes (`utils → database → functions → router`) construido
sobre `axum`, `sea-orm` (PostgreSQL), `redis`, `minio`, con `jsonwebtoken` +
`bcrypt` para la autenticación. Esta sección es solo una página de entrada; la
documentación completa está en [English](../en/README.md) o
[简体中文](../zhs/README.md).

---

## Índice de documentación

### Guías

| Guía | Contenido |
| --- | --- |
| [Detailed README](../en/guides/README.md) | Resumen del proyecto, stack técnico, inicio rápido |
| [Glossary](../en/guides/glossary.md) | Terminología de dominio chino-inglés |
| [Architecture](../en/guides/architecture.md) | Capas de los cuatro paquetes, flujo de peticiones, patrón `SafeEntityTrait` |
| [Building](../en/guides/building.md) | Requisitos previos, comandos `just`, archivo `.env`, docker-compose local |
| [API Reference](../en/guides/api-reference.md) | Dominios de API expuestos por el router (area/icon/item/marker/punctuate/score/system…) |
| [Commit Convention](../en/guides/commit-message-convention.md) | Convención de commits gitmoji |
| [Java Sync Roadmap](../en/guides/sync-with-java-roadmap.md) | Prioridad del portado desde la implementación Java |
| [Domain Sync Template](../en/guides/domain-sync-template.md) | Plantilla de cinco capas para portar un dominio Java a Rust |

### Diseños

- [Punctuate Workflow](../en/designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](../en/designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](../en/designs/hidden-and-special-flags.md)

---

## Otros idiomas

[简体中文](../zhs/README.md) · [English](../en/README.md) · [繁體中文](../zht/README.md) ·
[日本語](../ja/README.md) · [한국어](../ko/README.md) · [Français](../fr/README.md) ·
**Español** · [Русский](../ru/README.md) · [العربية](../ar/README.md) ·
[Deutsch](../de/README.md) · [Português](../pt/README.md)
