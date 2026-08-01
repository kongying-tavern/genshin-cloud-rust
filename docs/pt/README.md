# Genshin Map Cloud — Backend Rust

> A reescrita em Rust do backend « 空荧酒馆 Genshin Map », sincronizada com a
> implementação de referência em Java
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud)).

Esta é a seção de documentação em português. O backend é um workspace Cargo de
quatro pacotes (`utils → database → functions → router`) construído sobre
`axum`, `sea-orm` (PostgreSQL), `redis`, `minio`, com `jsonwebtoken` + `bcrypt`
para autenticação. Esta seção é apenas uma página de entrada; a documentação
completa está em [English](../en/README.md) ou
[简体中文](../zhs/README.md).

---

## Índice da documentação

### Guias

| Guia | Conteúdo |
| --- | --- |
| [Detailed README](../en/guides/README.md) | Visão geral do projeto, stack, início rápido |
| [Glossary](../en/guides/glossary.md) | Terminologia de domínio chinês-inglês |
| [Architecture](../en/guides/architecture.md) | Camadas dos quatro pacotes, fluxo de requisições, padrão `SafeEntityTrait` |
| [Building](../en/guides/building.md) | Pré-requisitos, comandos `just`, arquivo `.env`, docker-compose local |
| [API Reference](../en/guides/api-reference.md) | Domínios de API expostos pelo roteador (area/icon/item/marker/punctuate/score/system…) |
| [Commit Convention](../en/guides/commit-message-convention.md) | Convenção de commits gitmoji |
| [Java Sync Roadmap](../en/guides/sync-with-java-roadmap.md) | Prioridade do port da implementação Java |
| [Domain Sync Template](../en/guides/domain-sync-template.md) | Modelo de cinco camadas para portar um domínio Java para Rust |

### Designs

- [Punctuate Workflow](../en/designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](../en/designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](../en/designs/hidden-and-special-flags.md)

---

## Outros idiomas

[简体中文](../zhs/README.md) · [English](../en/README.md) · [繁體中文](../zht/README.md) ·
[日本語](../ja/README.md) · [한국어](../ko/README.md) · [Français](../fr/README.md) ·
[Español](../es/README.md) · [Русский](../ru/README.md) · [العربية](../ar/README.md) ·
[Deutsch](../de/README.md) · **Português**
