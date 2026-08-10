# Genshin Map Cloud — Rust 백엔드

> 「공영주점·원신 지도」백엔드의 Rust 재구현으로, Java 참조 구현
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud))
> 과 기능을 맞춥니다.

한국어 문서 섹션입니다. 백엔드는 `axum`·`sea-orm`(PostgreSQL)·`redis`·`minio`·
`jsonwebtoken` + `bcrypt`로 구성된 4패키지 Cargo 워크스페이스
(`utils → database → functions → router`)입니다. 이 섹션은 현재 진입
페이지이며, 전체 문서는 [English](../en/README.md) 또는
[简体中文](../zhs/README.md)을 참조하세요.

---

## 문서 색인

### 가이드

| 가이드 | 내용 |
| --- | --- |
| [Detailed README](../en/guides/README.md) | 프로젝트 개요·기술 스택·빠른 시작 |
| [Glossary](../en/guides/glossary.md) | 중영 도메인 용어집 |
| [Architecture](../en/guides/architecture.md) | 4패키지 계층·요청 흐름·`SafeEntityTrait` 패턴 |
| [Building](../en/guides/building.md) | 전제 조건·`just` 명령·`.env`·로컬 docker-compose |
| [API Reference](../en/guides/api-reference.md) | 라우터가 노출하는 API 도메인 (area/icon/item/marker/punctuate/score/system…) |
| [Commit Convention](../en/guides/commit-message-convention.md) | gitmoji 커밋 규약 |
| [Java Sync Roadmap](../en/guides/sync-with-java-roadmap.md) | Java 참조 구현 이식 우선순위 |
| [Domain Sync Template](../en/guides/domain-sync-template.md) | Java 도메인을 Rust로 이식하는 5계층 템플릿 |

### 설계

- [Punctuate Workflow](../en/designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](../en/designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](../en/designs/hidden-and-special-flags.md)

---

## 다른 언어

[简体中文](../zhs/README.md) · [English](../en/README.md) · [繁體中文](../zht/README.md) ·
[日本語](../ja/README.md) · **한국어** · [Français](../fr/README.md) ·
[Español](../es/README.md) · [Русский](../ru/README.md) · [العربية](../ar/README.md) ·
[Deutsch](../de/README.md) · [Português](../pt/README.md)
