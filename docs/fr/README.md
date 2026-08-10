# Genshin Map Cloud — Backend Rust

> La réécriture en Rust du backend « 空荧酒馆 Genshin Map », synchronisée avec
> l'implémentation de référence Java
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud)).

Ceci est la section documentaire en français. Le backend est un workspace Cargo
de quatre paquets (`utils → database → functions → router`) construit sur
`axum`, `sea-orm` (PostgreSQL), `redis`, `minio`, avec `jsonwebtoken` + `bcrypt`
pour l'authentification. Cette section n'est pour l'instant qu'une page
d'accueil ; la documentation complète est disponible en
[English](../en/README.md) ou [简体中文](../zhs/README.md).

---

## Index de la documentation

### Guides

| Guide | Contenu |
| --- | --- |
| [Detailed README](../en/guides/README.md) | Présentation, stack technique, démarrage rapide |
| [Glossary](../en/guides/glossary.md) | Terminologie de domaine chinois-anglais |
| [Architecture](../en/guides/architecture.md) | Couches des quatre paquets, flux de requête, patron `SafeEntityTrait` |
| [Building](../en/guides/building.md) | Prérequis, commandes `just`, fichier `.env`, docker-compose local |
| [API Reference](../en/guides/api-reference.md) | Domaines d'API exposés par le routeur (area/icon/item/marker/punctuate/score/system…) |
| [Commit Convention](../en/guides/commit-message-convention.md) | Convention de commits gitmoji |
| [Java Sync Roadmap](../en/guides/sync-with-java-roadmap.md) | Ordre de priorité du portage depuis l'implémentation Java |
| [Domain Sync Template](../en/guides/domain-sync-template.md) | Gabarit en cinq couches pour porter un domaine Java en Rust |

### Designs

- [Punctuate Workflow](../en/designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](../en/designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](../en/designs/hidden-and-special-flags.md)

---

## Autres langues

[简体中文](../zhs/README.md) · [English](../en/README.md) · [繁體中文](../zht/README.md) ·
[日本語](../ja/README.md) · [한국어](../ko/README.md) · **Français** ·
[Español](../es/README.md) · [Русский](../ru/README.md) · [العربية](../ar/README.md) ·
[Deutsch](../de/README.md) · [Português](../pt/README.md)
