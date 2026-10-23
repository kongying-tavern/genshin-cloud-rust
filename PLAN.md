# 迭代计划与维护指引

> 本文件只保留**仍然有效**的工作规则与前瞻事项。已完成的工作不再在此
> 登记——合并的 PR（squash 提交 + PR 描述）即变更日志，任意粒度用
> `git log` 过滤（2026-09-02 起与工作区规范对齐，见 `AGENTS.md` §3）。
> 历史上的转型步骤（2026-08 dev 收口）、里程碑 M1–M5、2026-09 安全与
> 工程审计整改（PR #133–#148）均已完结并清理出本文件。
> **定位（2026-09-27 起）**：Java 参考实现（`genshin-map-cloud`）已冻结、
> 基本不再变动——本仓库为唯一演进主轴。「与 Java 对齐」的约束降级为
> 「兼容既有线上 wire 契约」：前端在用的契约继续保持，内部实现可自由
> 演进；有意偏离 Java 的加固（如 WS 握手鉴权）不必再对照 Java 行为。

---

## 1. 分支模型与上游同步

```text
master  ── 唯一主线，永远可构建、CI 全绿；禁止直推、禁止 force-push
  ├── feat/<topic>     新功能
  ├── fix/<topic>      缺陷修复 / 技术债
  ├── test/<topic>     纯测试补充
  ├── docs/<topic>     纯文档
  ├── refactor/<topic> 重构
  └── chore/<topic>    基建 / CI / 依赖
```

- 分支从 **master 最新**切出，生命周期 ≤ 一个主题，合并后删除。
- `dev` 分支已退役（封存为 tag `archive/dev-snapshot` 备查）。
- 与上游 `kongying-tavern/genshin-cloud-rust` 的关系：**日常迭代全部在

own（langyo fork）master 进行**；阶段性攒批后向 upstream 发**同步 PR**
（`🔄` 标题，squash 合并）——同步分支的树与 own/master 严格一致
（`git diff` 为空），该模式自 upstream #50–#54 起已确立。upstream 历史
含中文提交不受本仓 lint 约束，同步 PR 内的提交保持自身规范即可。

## 2. PR 合并门禁（每个 PR 必须满足）

1. **单一主题**：一个 PR 只做一件事；大特性拆成可独立合并的小 PR。
1. **PR 标题 = gitmoji 格式**（`<gitmoji> <Capitalized English summary.>`）——

squash 合并时标题即 master 上的提交 subject，这是硬闸门。

1. **CI 全绿**：必需检查清单见 `AGENTS.md` §9（Build & Check、双 OS 测试、

DB integration、cargo-deny、Build image、commit-msg、Secrets Scan）。

1. **变更历史**：不维护 CHANGELOG 文件——合并的 PR 即变更日志，发布说明

写在 git tag + GitHub Releases。

1. **文档同步**：改动涉及行为/API 时，zh-Hans 与 en 文档**同 PR 更新**

（繁体 zh-Hant 亦应同步，防漂移）。

1. **测试**：新业务逻辑至少带 domain 级测试；涉及 SQL 的带 DB 集成测试。
1. **合并方式**：一律 **squash**——master 要求线性历史，且 merge-commit

的主题会被 commit-msg lint 拒绝。合并走 `celestia-devtools pr-merge`
或 `gh()` 代理函数，杜绝裸 `gh pr merge --squash` 绕过校验。

1. **自我 review**：提交前自己过一遍 diff（PR 模板 checklist 项）。

## 3. 提交信息强制（三层均已就位）

| 层 | 机制 |
| --- | --- |
| 本地 | `celestia-devtools hook install` 的 commit-msg hook（`just hooks` 重装） |
| CI | commit-msg.yml：PR 标题 + PR 内全部提交 + master push + merge_group 逐一 lint；force-with-lease 推送后 before SHA 不可达时降级为 lint 推送 tip |
| 合并 | `~/.bashrc` 的 `gh() { celestia-devtools gh "$@"; }` 代理（开发者本机一次性配置） |

## 4. 日常开发循环（每个补丁的标准动作）

```bash
git checkout master && git pull own master
git checkout -b fix/<topic>            # 或 feat/ test/ docs/ chore/
# ... 开发，提交会被本地 hook 校验 gitmoji 格式 ...
just ci                                 # 本地全量门禁：fmt-check + clippy + check + test
git push own fix/<topic>
gh pr create --repo langyo/genshin-cloud-rust --base master \
  --title "🐛 Fix ... ."               # 标题必须 gitmoji 格式
# CI 全绿 + checklist 自检后：
celestia-devtools pr-merge --squash --subject "🐛 Fix ... ." --repo langyo/genshin-cloud-rust
git checkout master && git pull own master && git branch -d fix/<topic>
```

## 5. 已知坑与注意事项

1. **Windows stdin 编码**：`git log | celestia-devtools ... --stdin-subjects`

在 Windows 下必须前置 `PYTHONUTF8=1`，否则 emoji 被 cp936 解码损坏造成
误报（CI/Linux 无此问题）。

1. **裸 `gh pr merge --squash` 绕过一切校验**——subject 直接成为合并提交。

必须走 devtools 代理（`gh()` 函数或 `celestia-devtools pr-merge`）。

1. **私有仓库分支保护收费**：本仓库 public，免费开启；勿转私有。
1. **Cargo.lock 与全局 `[patch]` 配置**：本机若有 celestia 工作区的全局

cargo patch，跑 cargo 会把 `[[patch.unused]]` 版本噪音写进 lock——提交前
`git diff --text Cargo.lock` 检查，噪音 hunk 剔除、真实依赖变更保留。

1. **分支保护的 up-to-date 要求**：master 前移后未 rebase 的 PR 会 BEHIND

阻塞合并——rebase 后 `--force-with-lease` 推送（lint 已对此降级处理）。

## 6. 常态维护指引

1. **schema 变更一律走新迁移文件**（`packages/migration`，baseline 冻结；

注册进 `Migrator`）。存量库升级前可先跑
`cargo run --bin init_db -- --check` 做漂移检测。

1. **WS 前端契约收敛**：已于 2026-10-23 定案——采用 socket.io 兼容模式

（路径 2：后端补 socket.io 端点、前端保持 socket.io-client），决策记录见
`docs/{en,zh-Hans,zh-Hant}/designs/ws-frontend-contract.md` 文末
「2026-10-23 决策」节。现存前端均不消费裸 `/ws` 端点，该端点与
`WS_AUTH_REQUIRED` 逃生阀暂保留，待 socket.io 通道落地后再议去留。

1. **本文件的维护纪律**：完成的事项随下次例行清理 PR 移出本文件，只增

不删会让这里重新变成历史堆积场；前瞻事项保持「仍然有效」这一个判据。

## 7. 前瞻事项（未排期）

1. **契约权威**：前端 `openapi2ts` 仍从 Java `/v3/api-docs` 生成客户端，而 Rust
   侧没有 OpenAPI 产物——每次契约变更都存在前端生成物与 Rust 实现漂移的风险。
   需二选一并配契约测试：Rust 侧生成 OpenAPI，或前端冻结生成物并加契约测试
   （未排期）。
1. **发布/生产状态记录义务**：`deploy/README.md` 的「当前线上归属」小节须在
   每次正式部署或环境交接后由维护者登记；未登记视为发布/生产状态未定。
