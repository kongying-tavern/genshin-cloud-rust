<!-- markdownlint-disable MD033 MD041 -->

# 空荧酒馆·原神地图 Rust 后端

> 简体中文文档索引 · [English](../en/README.md) · [繁體中文](../zht/README.md)

本项目是「空荧酒馆·原神地图」后端服务的 Rust 实现，目标是与 Java 参考实现
（[`java-genshin-map-cloud`](https://github.com/kongying-tavern/java-genshin-map-cloud)）
保持功能同步，同时在性能、部署体验与类型安全上有所改进。

技术栈基于 `axum` + `sea-orm` + `PostgreSQL` + `redis` + `minio`，工作区划分为
`utils / database / functions / router` 四个包，逐层从底向上依赖。鉴权由
`jsonwebtoken` + `bcrypt` 提供，运行时为 `tokio`，日志走 `tracing`。

## 指南 / Guides

| 文档 | 说明 |
| --- | --- |
| [详细 README](./guides/README.md) | 项目概述、技术栈、快速开始 |
| [领域术语表](./guides/glossary.md) | 中英文领域术语对照 |
| [架构概览](./guides/architecture.md) | 四包分层、请求流、`SafeEntityTrait`（乐观锁 + 软删除）与缓存集成点 |
| [构建指南](./guides/building.md) | 前置工具链、`just` 命令、`.env`、本地 `docker-compose` 与 CI |
| [API 参考](./guides/api-reference.md) | router 暴露的全部 API 域，按用途分组 |
| [提交规范](./guides/commit-message-convention.md) | celestia-devtools gitmoji 规范、钩子安装与跳过方式 |
| [Java 同步路线图](./guides/sync-with-java-roadmap.md) | Java 侧范围与七个移植优先级批次 |
| [域同步模板](./guides/domain-sync-template.md) | 单域移植的五层落地模式与 area 示例 |

## 设计文档 / Designs

- [打点审批工作流](./designs/punctuate-workflow.md) — 社区众包打点提交→审核→晋升的状态机设计
- [BinaryMD5 归档导出](./designs/binarymd5-archive-export.md) — 客户端冷启动的全量数据 GZIP 压缩增量同步管线
- [隐藏与特殊标记](./designs/hidden-and-special-flags.md) — `hidden_flag` 数据级防剧透 + `special_flag` 位掩码过滤

以下设计文档已编写完成。后续将记录 `rustls+ring` 加密后端选型、`sea-orm` 1.x→2.x
迁移、`SafeEntityTrait` 宏重写等关键决策。

## 快速入口

- 完整 README（含快速开始与许可证）：[详细说明](./guides/README.md)
- 目录（mdBook/lagrange 风格）：[SUMMARY](./SUMMARY.md)
- 顶层项目说明：[仓库根 README](../../README.md)

## 其他语言 / Other Languages

[English](../en/README.md) · [**简体中文**](./README.md) · [繁體中文](../zht/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Français](../fr/README.md) · [Español](../es/README.md) · [Русский](../ru/README.md) · [العربية](../ar/README.md) · [Deutsch](../de/README.md) · [Português](../pt/README.md)
