# 迭代模式转型与未完成事项迭代计划

> 盘点日期：2026-08-01 · 盘点基线：`dev` (a99f4e9) vs `master` (3b04ef0)
> 参照规范：[celestia-island/celestia-devtools](https://github.com/celestia-island/celestia-devtools) 的 master-based + PR 工作流
>
> **转型状态（2026-08-01 当日完成）**：§3 的 T0–T5 已全部执行完毕 ——
> 收口 PR #18 已 squash 合并进 master（22d5178）；CI 加固 PR #19 已合并
> （6d62915）；`dev` 已封存为 tag `archive/dev-snapshot`（8a4ef3e）并删除；
> master 分支保护已开启。本文件后续仅 §4 的里程碑 backlog 与 §5 的日常循环
> 继续有效。

---

## 0. 现状快照

| 项 | 状态 |
| --- | --- |
| 默认分支 | `master`（own 与 upstream 的 HEAD 均指向它） |
| `own/master` | 3b04ef0，领先 `upstream/master` 6 个提交（upstream 是其祖先，**无分叉**） |
| `dev` | a99f4e9，领先 `master` **54 个提交**，全部通过 gitmoji lint ✅ |
| 工作树 | 基本干净（仅 `Cargo.lock` 有实质改动，其余为 stat 脏标记） |
| master 分支保护 | **未配置**（GitHub API 404） |
| commit-msg 强制 | 本地 hook 已装 ✅；CI lint 已在 test.yml 但实现脆弱 ⚠️；分支保护缺失 ❌ |
| PR 模板 / issue 模板 / SECURITY / dependabot | 已存在 ✅ |

**结论**：当前实质是「dev 分支堆提交」模式。dev 上 54 个提交质量良好（lint 全过、
clippy 严格通过），具备一次性收口进 master 的条件。

---

## 1. 未完成事项清单（盘点结果）

### 1.1 功能缺口（代码内登记的 TODO / 占位实现）

| # | 位置 | 问题 | 严重度 |
| --- | --- | --- | --- |
| F1 | `router/routes/system/archive.rs:146` | rename 处理器是 stub：`auth` 被 `do_get_last` move，无法调 `do_rename`（已实现但成死代码）；需重构业务函数借用 `&AuthInfo` | 高 |
| F2 | `router/routes/system/archive.rs:182` | delete_slot 是 stub，缺 `do_delete_slot(user_id, slot_index)` | 高 |
| F3 | `functions/api/route.rs:115/145/161` | `do_get_page`/`do_get_search`/`do_get_list_by_id` 查询正确但丢弃结果，返回 `RouteEmptyResponse` 占位，待定义 `RouteVO` | 高 |
| F4 | `functions/api/score.rs` | `do_generate_score` 是简化聚合（按编辑次数计分），Java 字段级 diff（`ScoreDataPunctuateVo`）未移植；`do_get_score_data` 每行固定 score=1.0 | 高 |
| F5 | `functions/api/{item,marker,marker_link}_doc.rs` | BinaryMD5 端点无进程内缓存（Java 用 Caffeine），每请求重新生成；`list_page_bin` 首次 miss 全表扫描；MD5 清单 `time` 用请求时间戳而非数据变更时间 | 高（性能） |
| F6 | `functions/api/punctuate*.rs` | `do_pass`/`do_reject` **无角色权限校验**；晋升「写 marker + 删 punctuate」两条独立 SQL 无事务 | 高（安全） |
| F7 | `functions/system/oauth.rs:112/177` | `access_policy` 未做检查；scope 字符串未映射枚举（固定 `All`）；JWKS 公钥分发端点缺失；QQ 第三方登录是占位 | 高（安全） |
| F8 | `functions/system/user.rs` | 注册硬编码 `"default_password"`；`do_update_password` 不校验旧密码；`do_list` 的 sort 参数未应用；`do_kick_out` 空实现 | 中 |
| F9 | `functions/api/marker.rs:129` | `do_tweak` 的 `ItemList` 分支「逻辑复杂此处跳过」 | 中 |
| F10 | `router/routes/api/cache/*` | 7 个缓存刷新端点全部是 no-op（Redis 缓存层未接线） | 中 |
| F11 | `database/models/marker/marker_linkage.rs` 等 | 5 列「其实不能为空」待更正表关系；`sys_user_archive` 未绑定完整存档结构；`sys_user_device` 数据存疑 | 低（需真实库验证） |

### 1.2 测试缺口

| # | 问题 |
| --- | --- |
| T1 | 除 area/marker 的 schema 级断言外，**所有域无测试**（user/role/device/invitation/archive/action_log/oauth/punctuate/score/history/notice/route/icon/item/tag/marker_link/cache/res） |
| T2 | `tests/docker/docker-compose.e2e.yml` 声称服务于 `#[ignore]` 的 DB 测试，但**全仓库没有任何 `#[ignore]` 测试**——compose 无消费者 |
| T3 | Python e2e 仅 5 个冒烟用例，API 返回 401/403 也算通过，无业务断言 |
| T4 | CI 不起 DB/Redis/MinIO 服务，补上 DB 测试会直接失败 |

### 1.3 基建缺口

| # | 问题 |
| --- | --- |
| I1 | **Dockerfile 损坏**：重复 `cargo new`、缺 `COPY --from` 产物、`ENTRYPOINT ["./a"]` 指向不存在的二进制（实际名 `_router`）、混入无关的 wasm32 target 与 cargo-make |
| I2 | `deny.toml` 已配置但**无 cargo-deny CI** |
| I3 | 无 e2e CI（`scripts/e2e` 与 compose 栈均不进工作流） |
| I4 | `/cdn` 反代硬编码 `v3.yuanshen.site` + 本地伪造空 dadian 配置（开发期临时方案，需可配置化） |
| I5 | test.yml 的 commit-msg job 用 `github.event.push.commits[0].size` 做 fallback，PR 事件下该字段不存在，逻辑脆弱；且未 lint PR title（squash 合并的真正闸门） |

### 1.4 文档缺口

| # | 问题 |
| --- | --- |
| D1 | zhs roadmap 无 Status 列且「跟进事项」把**已完成**的 sea-orm 2.x / minio 0.4 迁移列为待办；en 版有 Status 列但把 4/6/7 标 Planned（实际已有简化实现）——两版漂移且均低估进度 |
| D2 | 11 个语言目录中仅 en/zhs 有完整 14 篇，其余 9 种仅骨架 |
| D3 | CHANGELOG `Unreleased` 段落随 dev 堆叠，未按迭代切分 |

---

## 2. 目标迭代模式（参照 celestia-devtools）

### 2.1 分支模型

```
master  ── 唯一主线，永远可构建、CI 全绿；禁止直推、禁止 force-push
  ├── feat/<topic>    新功能
  ├── fix/<topic>     缺陷修复 / 技术债
  ├── test/<topic>    纯测试补充
  ├── docs/<topic>    纯文档
  ├── refactor/<topic> 重构
  └── chore/<topic>   基建 / CI / 依赖
```

- 分支从 **master 最新**切出，生命周期 ≤ 一个主题，合并后删除。
- `dev` 分支**退役**：收口 PR 合并后删除远端 dev（本地留 tag `archive/dev-snapshot` 备查）。
- 与上游 `kongying-tavern` 的关系：**日常迭代在 own（langyo fork）master 进行**；
  向上游贡献作为独立里程碑单独决策（上游历史含中文提交、未执行 gitmoji 规范，
  直接 PR 到上游会被我们自己的 lint 拦，需先与上游对齐规范或豁免）。

### 2.2 PR 合并门禁（每个 PR 必须满足）

1. **单一主题**：一个 PR 只做一件事；大特性拆成可独立合并的小 PR。
2. **PR 标题 = gitmoji 格式**（`<gitmoji> <Capitalized English summary.>`）——
   squash 合并时标题即 master 上的提交 subject，这是硬闸门。
3. **CI 全绿**：fmt-check、clippy `-D warnings`、cargo check、cargo test、
   commit-msg lint（含 PR title）、secrets scan。
4. **变更历史**：不维护 CHANGELOG 文件（2026-09-02 起与工作区规范对齐，
   见 `AGENTS.md` §3）——合并的 PR（squash 提交 + PR 描述）即变更日志，
   发布说明写在 git tag + GitHub Releases。
5. **文档同步**：改动涉及行为/API 时，zhs 与 en 文档**同 PR 更新**（防止 D1 类漂移）。
6. **测试**：新业务逻辑至少带 domain 级测试；涉及 SQL 的带 DB 集成测试（M1 基建就位后强制）。
7. **合并方式**：默认 **squash**；仅大型多提交特性（每 commit 均过 lint）允许 merge commit。
   合并一律走 `celestia-devtools pr-merge` 或 `gh()` 代理函数，杜绝裸 `gh pr merge --squash` 绕过校验。
8. **自我 review**：提交前自己过一遍 diff（PR 模板 checklist 项）。

### 2.3 提交信息三层强制（对齐 devtools）

| 层 | 机制 | 现状 | 动作 |
| --- | --- | --- | --- |
| 本地 | `celestia-devtools hook install` 的 commit-msg hook | ✅ 已装 | 保持；`just hooks` 可重装 |
| CI | reusable workflow `celestia-island/celestia-devtools/.github/workflows/commit-msg-lint.yml@master`（lint PR title + PR 内全部 commit） | ⚠️ test.yml 内自写脆弱版 | **替换为 reusable 版**（见 3-T3） |
| 合并 | `~/.bashrc` 加 `gh() { celestia-devtools gh "$@"; }` 代理 | ❌ | 开发者本机配置（一次性） |

### 2.4 master 分支保护（合并收口 PR 后立即开启）

- Require a pull request before merging（approvals: 0，单人项目可先自审；协作后调 1）
- Require status checks：`Rust / fmt+clippy+check+build`、`Test / test (ubuntu+windows)`、
  `Commit Message Lint`、`Secrets Scan`
- Require linear history（配合 squash 默认）
- Block force pushes & deletions
- （public 仓库免费；私有需 Team 版——本仓库 public 无此顾虑）

---

## 3. 转型执行步骤（一次性，按顺序）—— ✅ 已全部完成（2026-08-01）

| 步 | 动作 | 实际执行结果 |
| --- | --- | --- |
| T0 | 处理工作树遗留的 `Cargo.lock` 改动 | ✅ 判定为外部 `[patch]` 上下文污染的 `[[patch.unused]]` 条目，直接 discard |
| T1 | 收口 PR：`dev` → `master` | ✅ [PR #18](https://github.com/langyo/genshin-cloud-rust/pull/18)，按用户决定以 **squash** 合并（22d5178）。合并前顺带修复了 3 个既有 CI bug：① sccache 手工安装脚本引用了错误的解压目录名（v0.15.0 起始终失败，换 `mozilla-actions/sccache-action@v0.0.11`）；② 全局 `RUSTC_WRAPPER=sccache` 使 Windows job 因无 sccache 崩溃（action 双平台安装解决）；③ trufflehog `extra_args` 重复 `--fail` 被拒（去掉） |
| T2 | dev 封存 | ✅ tag `archive/dev-snapshot`（8a4ef3e）已推送，远端/本地 `dev` 已删除 |
| T3 | CI 加固 PR | ✅ [PR #19](https://github.com/langyo/genshin-cloud-rust/pull/19)（6d62915）：commit-msg 换 devtools reusable workflow（PR title + commits 双 lint，check 名 `Commit Message Lint / Lint commit messages`）；新增 `deny.yml`（cargo-deny 四查）；deny.toml 补 4 个宽松许可证（bzip2-1.0.6 / NCSA / CDLA-Permissive-2.0 / BSL-1.0）并有依据地忽略 RUSTSEC-2023-0071（rsa 仅 HMAC 链路传递依赖、从未被调用）；rust/test/docs 三 workflow 触发分支去掉 `dev` |
| T4 | master 分支保护 | ✅ 已开启：require PR + 6 个必需状态检查 + linear history + 禁 force-push/删除 |
| T5 | 本机 gh 代理 | ✅ `~/.bashrc` 已配 `gh() { celestia-devtools gh "$@"; }` |
| T6 | 文档更新 | ✅ 本 PR（docs/iteration-plan）：PLAN.md 入库、README 增加迭代工作流节、CHANGELOG 记录转型 |

> 仓库已正式进入「任何补丁 = 主题分支 + PR → master」模式。

---

## 4. 迭代里程碑 backlog

> 每行 = 一个独立 PR（分支名 + 拟定 PR 标题 + 验收 DoD）。里程碑内可并行，跨里程碑有依赖。

### M1 — 基建修复与测试底座（先行，后续迭代的门槛） — ✅ 已完成（PR #22–24）

| 分支 | PR 标题 | 内容 | DoD |
| --- | --- | --- | --- |
| `chore/dockerfile-rewrite` | `🔨 Rewrite the Dockerfile as a working multi-stage build.` | 删重复 `cargo new`/wasm 残留；正确 COPY `_router` 产物；ENTRYPOINT 修正 | `docker build` 成功、容器启动健康检查通过 |
| `chore/cargo-deny-ci` | `💚 Add the cargo-deny workflow for license and advisory checks.` | deny.toml 接入 CI | workflow 绿 |
| `test/db-integration-harness` | `✅ Wire the docker compose stack into DB-backed integration tests.` | compose(PG/Redis/MinIO) 接 CI service 或 job 容器；写第一个 user 域 DB 测试作为样板；T2 的注释与现实对齐 | CI 起服务并跑通 ≥1 个 `#[ignore]`→正式测试 |
| `test/e2e-business-assertions` | `✅ Upgrade e2e from smoke checks to authenticated business assertions.` | e2e 带登录态；401/403 不再算通过；覆盖 area/marker/item_doc 至少 3 条业务断言 | `just dev mock` 绿且断言真实数据 |

### M2 — 技术债清零（CHANGELOG 已登记的 5 条 + 占位实现） — ✅ 已完成（PR #25–29）

| 分支 | PR 标题 | 内容 | DoD |
| --- | --- | --- | --- |
| `fix/archive-rename-borrow` | `🐛 Refactor archive functions to borrow AuthInfo and wire rename.` | F1：`&AuthInfo` 重构，接通 `do_rename` | rename 端点真实生效 + 测试 |
| `fix/archive-delete-slot` | `✨ Add the archive delete_slot operation.` | F2：实现 `do_delete_slot` | 端点生效 + 测试 |
| `feat/route-vo` | `✨ Define RouteVO and return real route page/search/list data.` | F3：RouteVO 定义，替换三处 `RouteEmptyResponse` | 三端点返回真实数据 + 测试 |
| `fix/user-domain-placeholders` | `🐛 Remove the user domain placeholder implementations.` | F8：注册默认密码、旧密码校验、sort 生效、kick_out 实现 | 四处行为对齐 Java + 测试 |
| `fix/marker-tweak-item-list` | `🐛 Implement the ItemList branch of marker tweak.` | F9 | 行为对齐 Java + 测试 |

### M3 — 权限与安全（生产部署前必须） — ✅ 已完成（PR #30–33）

| 分支 | PR 标题 | 内容 | DoD |
| --- | --- | --- | --- |
| `feat/punctuate-audit-authz` | `🔒 Enforce role checks and transactional promotion in punctuate audit.` | F6：pass/reject 加角色中间件；晋升事务化 | 无权限请求被拒；事务回滚可验证 |
| `feat/oauth-access-policy` | `🔒 Enforce oauth access_policy checks and scope mapping.` | F7 前半：access_policy 检查 + scope 映射 | 对齐 Java 行为 + 测试 |
| `feat/jwks-endpoint` | `✨ Add the JWKS public key distribution endpoint.` | F7 后半：JWKS 路由 + 密钥轮换策略 | `/.well-known/jwks.json` 可用 |
| `feat/qq-login` | `✨ Implement QQ third-party login.` | F7/F8 交集：`do_register_qq` 真实现 | 全流程可登录（mock QQ 侧） |

### M4 — 性能与缓存 — 🟡 部分完成（moka 进程内缓存 + 刷新接线 PR #35–36；Redis 二级缓存未做）

| 分支 | PR 标题 | 内容 | DoD |
| --- | --- | --- | --- |
| `feat/binarymd5-cache` | `⚡ Add the two-tier cache for BinaryMD5 doc endpoints.` | F5：进程内缓存（moka）+ Redis 二级；MD5 `time` 改数据变更时间 | 重复请求不再重生成；失效正确 |
| `feat/cache-refresh-wiring` | `✨ Wire the cache refresh endpoints to the Redis cache layer.` | F10：7 个 no-op DELETE 端点接线 | 刷新后缓存确实失效 + 测试 |

### M5 — 文档同步与首个发布 — ✅ 已完成（PR #37–39，v0.2.0）

| 分支 | PR 标题 | 内容 | DoD |
| --- | --- | --- | --- |
| `docs/roadmap-resync` | `📝 Resync the zhs and en roadmaps with actual domain status.` | D1：两版加统一 Status 列、删过时跟进事项 | 两版内容一致 |
| `docs/cdn-proxy-config` | `📝 Make the CDN proxy upstream configurable and document it.` | I4：/cdn 上游与 dadian 注入可配置化 + 文档 | 无硬编码域名 |
| `chore/release-0.2.0` | `🚀 Release version 0.2.0.` | CHANGELOG 按 M1–M4 切分 0.2.0 段落；workspace version 提升 | tag + release notes |

> F11（DB schema 修正）依赖真实数据库数据验证，单列到 M5 之后的「数据对齐」专题，
> 不在本计划排期内。D2（9 语言翻译）按 i18n/* 分支逐个语言跟进，不阻塞迭代。

---

## 5. 日常开发循环（转型后每个补丁的标准动作）

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

---

## 6. 已知坑与注意事项

1. **Windows stdin 编码**：`git log | celestia-devtools ... --stdin-subjects` 在 Windows
   下必须前置 `PYTHONUTF8=1`，否则 emoji 被 cp936 解码损坏造成误报（CI/Linux 无此问题）。
2. **裸 `gh pr merge --squash` 绕过一切校验**——subject 直接成为合并提交。必须走
   devtools 代理（`gh()` 函数或 `celestia-devtools pr-merge`）。
3. **私有仓库分支保护收费**：本仓库 public，免费开启；勿转私有。
4. **向上游 PR 的 lint 冲突**：upstream 历史有中文提交；给上游提 PR 时其 base 不含
   lint 则无妨，但我们侧 CI 会 lint PR 内 commit——保持我们提交规范即可兼容。
5. **收口 PR 用 merge commit 不用 squash**：54 个提交已全部合规，squash 只会丢失粒度。
