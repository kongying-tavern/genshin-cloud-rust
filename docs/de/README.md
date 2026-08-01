# Genshin Map Cloud — Rust-Backend

> Die Rust-Neuimplementierung des „空荧酒馆 Genshin Map“-Backends, funktionsgleich
> mit der Java-Referenzimplementierung
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud)).

Dies ist der deutschsprachige Dokumentationsbereich. Das Backend ist ein
Cargo-Workspace aus vier Paketen (`utils → database → functions → router`) auf
Basis von `axum`, `sea-orm` (PostgreSQL), `redis`, `minio` sowie
`jsonwebtoken` + `bcrypt` für die Authentifizierung. Dieser Bereich ist derzeit
nur eine Einstiegsseite; die vollständige Dokumentation finden Sie auf
[English](../en/README.md) oder [简体中文](../zhs/README.md).

---

## Dokumentationsindex

### Anleitungen

| Anleitung | Inhalt |
| --- | --- |
| [Detailed README](../en/guides/README.md) | Projektübersicht, Technologie-Stack, Schnellstart |
| [Glossary](../en/guides/glossary.md) | Chinesisch-englische Fachbegriffe |
| [Architecture](../en/guides/architecture.md) | Schichtung der vier Pakete, Request-Fluss, `SafeEntityTrait`-Muster |
| [Building](../en/guides/building.md) | Voraussetzungen, `just`-Befehle, `.env`, lokales docker-compose |
| [API Reference](../en/guides/api-reference.md) | Vom Router bereitgestellte API-Domänen (area/icon/item/marker/punctuate/score/system…) |
| [Commit Convention](../en/guides/commit-message-convention.md) | gitmoji-Commit-Konvention |
| [Java Sync Roadmap](../en/guides/sync-with-java-roadmap.md) | Portierungsreihenfolge aus der Java-Referenz |
| [Domain Sync Template](../en/guides/domain-sync-template.md) | Fünf-Schichten-Vorlage zum Portieren einer Java-Domäne nach Rust |

### Designs

- [Punctuate Workflow](../en/designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](../en/designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](../en/designs/hidden-and-special-flags.md)

---

## Andere Sprachen

[简体中文](../zhs/README.md) · [English](../en/README.md) · [繁體中文](../zht/README.md) ·
[日本語](../ja/README.md) · [한국어](../ko/README.md) · [Français](../fr/README.md) ·
[Español](../es/README.md) · [Русский](../ru/README.md) · [العربية](../ar/README.md) ·
**Deutsch** · [Português](../pt/README.md)
