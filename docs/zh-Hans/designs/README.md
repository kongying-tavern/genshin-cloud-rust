<!-- markdownlint-disable MD033 MD041 -->

# 设计文档索引

> [← 返回总索引](../README.md) · [架构概览](../guides/architecture.md)

空荧酒馆·原神地图是一个**众包贡献型社区地图**：神瞳、宝箱、奇馈宝箱、地笼、
采集物等点位主要由玩家贡献，编辑团队审核后才进入正式数据。这套「人人可打点、
人人可见、但需把关」的协作模型，催生了若干在普通增删改查后端里看不到的设计
决策。本目录收录这些决策的设计文档，重点说明**为什么**在原神互动地图的业务
语境下要这么做，而不是把通用后端模板照搬过来。

每篇文档大致包含三部分：背景与动机（这个数据结构 / 管线为何存在）、对 Java
参考实现的对齐情况（`genshin-map-cloud`）、以及 Rust 侧当前的落地状态
（含已知简化与后续待办）。

## 文档列表

| 文档 | 主题 | 核心问题 |
| --- | --- | --- |
| [BinaryMD5 归档导出](./binarymd5-archive-export.md) | `*_doc` GZIP 压缩批量导出管线 | 客户端冷启动如何快速拉取数千个 POI，且只同步变更页 |
| [隐藏标记与特殊标记](./hidden-and-special-flags.md) | `hidden_flag` / `special_flag` / `del_flag` 三套正交标记 | 防剧透、测试服隔离、UI 过滤、软删除如何互不干扰 |

## 为什么单独成文

这两块是整个 Rust 后端里最「业务驱动」的部分，恰恰也是最容易在新人 review
时被误改成「看起来更通用、实际破坏了游戏语义」的部分：

- BinaryMD5 的「按 `id / 3000` 分页 + MD5 寻址」看似是普通的分页缓存，
  实则和客户端的增量同步协议强绑定，改分页粒度会同时动到前后端；
- `hidden_flag` 与 `special_flag` 名字相近但语义完全不同（一个是**谁能看**，
  一个是**查不查得出来**），且都和 `del_flag` 软删除正交，混用会直接把测试服
  数据漏给正式服玩家。

因此把它们从 [架构概览](../guides/architecture.md) 里独立出来，作为可被
链接、可被 review 引用的稳定参考。

## 相关资料

- Java 参考实现：[`genshin-map-cloud`](https://github.com/kongying-tavern/genshin-map-cloud)
- Rust 包结构：见 [架构概览](../guides/architecture.md#四包分层) 的四包分层
- 领域术语对照（神瞳 / 宝箱 / 锄地等）：见 [领域术语表](../guides/glossary.md)
- Java → Rust 同步进度：见 [Java 同步路线图](../guides/sync-with-java-roadmap.md)
