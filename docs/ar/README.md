# Genshin Map Cloud — الخادم الخلفي بلغة Rust

> إعادة كتابة الخادم الخلفي «空荧酒馆 Genshin Map» بلغة Rust، بميزات متزامنة
> مع التنفيذ المرجعي بلغة Java
> ([`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud)).

هذا هو قسم التوثيق باللغة العربية. الخادم الخلفي هو مساحة عمل Cargo من أربع
حزم (`utils → database → functions → router`) مبنية على `axum` و`sea-orm`
(PostgreSQL) و`redis` و`minio`، مع `jsonwebtoken` + `bcrypt` للمصادقة. هذا
القسم هو صفحة دخول فقط حاليًا؛ التوثيق الكامل متاح باللغة
[الإنجليزية](../en/README.md) أو [الصينية المبسطة](../zhs/README.md).

---

## فهرس التوثيق

### الأدلة

| الدليل | المحتوى |
| --- | --- |
| [Detailed README](../en/guides/README.md) | نظرة عامة على المشروع وتقنياته وبدء سريع |
| [Glossary](../en/guides/glossary.md) | مصطلحات المجال (صيني-إنجليزي) |
| [Architecture](../en/guides/architecture.md) | طبقات الحزم الأربع وتدفق الطلبات ونمط `SafeEntityTrait` |
| [Building](../en/guides/building.md) | المتطلبات وأوامر `just` وملف `.env` وdocker-compose المحلي |
| [API Reference](../en/guides/api-reference.md) | مجالات API التي يعرضها الموجّه (area/icon/item/marker/punctuate/score/system…) |
| [Commit Convention](../en/guides/commit-message-convention.md) | اصطلاح الالتزامات gitmoji |
| [Java Sync Roadmap](../en/guides/sync-with-java-roadmap.md) | أولويات النقل من تنفيذ Java |
| [Domain Sync Template](../en/guides/domain-sync-template.md) | قالب من خمس طبقات لنقل مجال Java إلى Rust |

### التصميمات

- [Punctuate Workflow](../en/designs/punctuate-workflow.md)
- [BinaryMD5 Archive Export](../en/designs/binarymd5-archive-export.md)
- [Hidden and Special Flags](../en/designs/hidden-and-special-flags.md)

---

## لغات أخرى

[简体中文](../zhs/README.md) · [English](../en/README.md) · [繁體中文](../zht/README.md) ·
[日本語](../ja/README.md) · [한국어](../ko/README.md) · [Français](../fr/README.md) ·
[Español](../es/README.md) · [Русский](../ru/README.md) · **العربية** ·
[Deutsch](../de/README.md) · [Português](../pt/README.md)
