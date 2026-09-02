# AGENTS.md — genshin-cloud-rust 仓库级 AI 代理工作规则

> 本文件改编自工作区级规范 `/mnt/codespace/AGENTS.md`（celestia 工作区，2026-09-02
> 版），收录其中**与本项目相关的全部规则**并按本仓库实际情况适配。所有在本仓库
> 工作的 subagent 与 AI 工具必须遵守本文件。
>
> 工作区基础设施章节（大文件下载纪律 §0.6、节点访问表 §1、编译纪律与 malkuth
> §7.2、node-2/3 部署 §8、pnpm 前端家族与 worktree 软链 §9）不适用于本仓库，
> 仍以工作区文件为准。**工作区文件中的节点凭据表按 §8 红线严禁复制进任何仓库
> 文件（含本文件）。**

---

## 1. 仓库定位与双远程

- 空荧酒馆·原神地图 Rust 后端（Genshin Map Cloud Rust）。纯 Rust workspace：
  `packages/{database,functions,router,utils}` + `tests/rust`（含 DB 集成测试），
  无前端包、无跨仓 path/git 依赖。
- 远程：
  - `own` = `langyo/genshin-cloud-rust` —— **工作 PR 的目标**，master 受分支保护
    （必需检查 + 线性历史 + 禁 force push）。
  - `upstream` = `kongying-tavern/genshin-cloud-rust` —— 官方仓。日常改动先合并
    进 `own/master`，阶段性攒批后向 upstream 发同步 PR。
- 生产部署走 Docker（GHCR 镜像）+ `deploy/deploy.sh`，不涉及工作区节点。

## 2. Commit Message Format

```
<gitmoji> <Capitalized English summary ending with period.>
```

- 必须以一个 gitmoji 开头。白名单即 CI linter（`celestia-devtools commit-msg-lint`）
  强制的那套：gitmoji.dev 规范集加组织增补 🔗（symlink/copilot）🔄（sync/refresh）
  📜（license）🛡️（shield）。常用：✨ 🐛 🔧 ♻️ 🔥 📝 🎨 ✅ 🚀 🌐 ⬆️ 🎉 📦 🔒 ⬇️ 🚑。
- 摘要用英文、首字母大写、以 `.` 结尾。
- 禁止 CJK 字符；禁止 `fix:` / `feat:` 之类前缀。
- **摘要是描述改动的一句话**。禁止任何 `xxx:` / `xxx(scope):` 冒号前缀形态——包括
  句首短语（`🔧 Fix compliance: nonce handshake` ❌；`🔧 Fix nonce handshake and
  embed path.` ✅），也包括 `Topic phrase: details` 形态（`♻️ Audit round 23: drop
  dead queries.` ❌）。gitmoji 已表达改动类型，摘要中不得再用 "type:" 复述。
  详细上下文放 commit BODY（空行 + 列表），绝不放摘要行。
- 禁止 "Merge branch xxx" —— 一律 squash merge，merge-commit 主题会被 CI linter 拒绝。
- `git revert` 产生的 `Revert "..."` 主题豁免 gitmoji 要求（与 CI linter 一致）。
- **PR 标题遵循完全相同的规则**：`<gitmoji> <一句话描述.>`，无冒号前缀。squash
  合并后的主题 = PR 标题 + ` (#PRID)`。
- CI 强制：`.github/workflows/commit-msg.yml` 对 PR 标题、PR 内每个提交、master
  新提交、merge-group 提交逐一 lint。

## 3. CHANGELOG Policy（2026-09-02 起强制）

- **本仓库不维护任何 CHANGELOG / 修订历史文件。** 合并的 PR 即变更日志：squash
  提交（gitmoji + 一句话摘要）+ PR 描述构成完整变更历史，任意粒度用 `git log`
  过滤即可。手工维护的 changelog 文件必然漂移过期。
- 发布说明写在 **git tag + GitHub Releases** 页（按发布写，不按提交写），不落在
  任何被跟踪的文件里。
- 本仓库原 CHANGELOG.md 已于 2026-09-02 删除；不要重建，初始化新仓库/新目录时
  也不要添加。若 PR 模板或 workflow 仍引用 changelog，在同 PR 内一并移除该引用。

## 4. PR Workflow

每一段工作遵循固定模式：

1. **从 master 切特性分支**（`feat/<name>` / `fix/<name>` / `chore/<name>` /
   `refactor/<name>`），在隔离 checkout（git worktree 或独立克隆）中工作，
   不与其他任务共用手头的主 checkout。
2. **3 轮验证循环**：对每处改动——
   - 第 1 轮：分析 → 改进 → 验证；
   - 第 2 轮：再分析 → 改进 → 验证；
   - 第 3 轮：终审 → 打磨 → 验证。
   **任一轮失败即从零重新计 3 轮。**
3. 以 gitmoji 格式**提交**（见 §2）。
4. **推送**分支到 `own`。
5. `gh pr create -R langyo/genshin-cloud-rust`（gh 默认解析到 upstream，必须显式
   指定 `-R`）。
6. **Squash merge**（自主合并条件见 §6）：squash 主题 = `<gitmoji> <一句话.> (#PRID)`。
7. 合并后**删除**特性分支。

Subagent 使用纪律：

- 所有非平凡任务用 subagent（general / explore）执行，避免上下文污染与记忆串味。
- 给 subagent 的任务描述必须完整：要读写的确切路径、遵循的惯例（先看现有代码
  模式）、验收标准、commit 消息格式。
- 相互独立的子任务并行发起 subagent；有依赖的串行链式推进。
- 每个 subagent 返回前必须自验自己的工作；交叉验证由另一个 subagent 复核。
- `cargo check`（或相应构建）必须在提交前通过。

## 5. Branch Naming & Git Push Rules

- `master` —— 生产分支，受保护，只接受 squash merge。
- `feat/<name>` 新功能；`fix/<name>` 修 bug；`chore/<name>` 维护性改动；
  `refactor/<name>` 无行为变化的重构。
- `dev` —— **已废弃（本仓库已封存为 tag `archive/dev-snapshot` 并删除），禁用。**

推送纪律（硬性，无例外）：

- **绝不裸 `git push --force`**，除非用户显式授权。
- feature 分支上 rebase/amend 恢复一律优先 `git push --force-with-lease`。
- `--force-with-lease` 被拒（远端跟踪引用过期）时**立即停下，绝不回退到
  `--force`**：先 fetch，用 `git log origin/<branch>..HEAD` 与
  `git log HEAD..origin/<branch>` 审查两端提交，确认没有未知提交后再问用户。
- **master 上任何形式的 force push 绝对禁止**（分支保护亦已禁止）。master 只经
  squash merge 前进。
- 拿不准就别 force——开新分支、重新提交、或问用户。

## 6. Merge & Release Rules

- **允许自主合并 PR**（无需逐 PR 人工确认），前提是满足 §2、§5 及：
  1. **消息合规**：squash 主题 = `<gitmoji> <一句英文句号结尾.>`，无冒号前缀；
     PR 标题同规则。
  2. **检查门槛**：必需检查通过后才合并。必需检查因**环境性原因**失败（runner
     故障、外部服务不可达、网络阻断等）时，仅当失败已记录在 PR 描述/评论中、且
     改动通过本地验证（`cargo test` / lint）后方可豁免合并。**绝不在真实代码
     失败（编译/测试/clippy）上合并。**
  3. **PR 经济**：不要为每个琐碎改动单开 PR 即刻合并——PR 号是有限资源。一个 PR
     应捆绑一批可合并的功能（一个连贯的功能/修复波次，最好含多个相关提交）。
     只有确实无可捆绑（紧急 hotfix、无关联待合并工作的单条规则/CLI 改动）时才
     允许小 PR。
- **版本号随主 PR 走**：`Cargo.toml` 版本提升放进功能/修复 PR 本身，不单开版本
  bump 补丁 PR（除非用户明确要求）。
- **不在获批工作流之外建 PR**：只在被明确要求或作为获批工作流步骤（如 PLAN.md
  任务执行）时创建 PR；自发/计划外 PR 仍需先获许可。

## 7. Build & Test

- Rust 门禁：`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets
  -- -D warnings`、`cargo check --workspace --all-targets`、`cargo test --workspace`。
  `just ci` 一键跑全套。
- DB 集成测试本地默认自跳过（未设 `GCS_TEST_DB` 时）；要真正跑起来用
  `tests/docker/docker-compose.e2e.yml` 起库，或看 CI 的 DB integration job。
- 若未来引入跨仓依赖，一律用 **git 引用**（`branch = "master"`），禁止本地
  path 依赖进 Cargo.toml。
- 提交前 `cargo check` 必须通过（§4）。

## 8. 敏感信息红线（硬性，违反即视为事故）

> 背景：工作区曾因真实 SSH 密码入库被迫 filter-repo 重写历史 + 全部 tag 重映射，
> 代价巨大、不可撤销。

1. **禁止把任何真实密码 / 密钥 / token / 内网 IP 写进 git 树**——任何分支、任何
   文件，包括注释、示例、默认值、测试数据、README、docs。
2. 代码里需要密码时：用环境变量 / 不入库的配置文件，或占位符（`<your-password>`
   / `CHANGE_ME`）；示例地址一律用 RFC 5737 文档段（192.0.2.x / 198.51.100.x /
   203.0.113.x），示例值用明显的假值（`test-password` / `sk-xxx`）。
3. 确有必要写真实凭据的极少数情况：**先问用户**，并评估仓库可见性（公开仓 ≠
   仅自己可见，历史泄漏不可撤销）。
4. **提交前自查**：涉及配置 / 部署 / install 脚本 / 示例数据的改动，grep 一遍
   `password|secret|token|api_key` 确认无真实值；内网 IP（192.168.x / 10.x）用
   文档地址替代。
5. 工作区本地文件（`/mnt/codespace/AGENTS.md`、`PLAN.md` 等）中的真实凭据**只准
   留在本地**，禁止复制进任何仓库文件（含仓库级 AGENTS.md，即本文件）。
6. 泄漏处置：a) 立即从当前分支/PR 删除；b) 评估泄漏面（tag / 分支 / 下游引用）；
   c) 报告用户，由用户决定是否历史重写（涉 master force push 需显式授权）；
   d) 无论是否重写，凭据视为已公开，**必须轮换**。

## 9. CI 使用策略（本仓库适配）

- 本仓库全部使用 **GitHub-hosted runner**（无自建池配额约束）。因此 PR 每次
  push（synchronize）都触发全部检查是**有意保留的策略**（2026-09-02 用户确认）；
  工作区级「PR 仅 opened/reopened/ready_for_review 触发 + 手动 dispatch」策略
  针对自建 runner 稀缺，**不适用于本仓库**。
- **CI 是参考不是门禁**：合并前看一眼 checks 有没有**代码级失败**（编译/测试/
  clippy）；有则修，没有（环境性失败 / 排队）且本地验证已绿即可按 §6.2 豁免
  合并，不要长时间盯 CI。
- 各 workflow 均已带 `concurrency` + `cancel-in-progress`；不再需要的重复 run
  （尤其重复 push 触发的旧 run）用 `gh run cancel <id> -R langyo/genshin-cloud-rust`
  清理。
- master 分支保护必需检查：`Build & Check`、`Test (ubuntu-latest)`、
  `Test (windows-latest)`、`DB integration`、`cargo-deny`、`Build image`、
  `Commit Message Lint / Lint commit messages`、`Secrets Scan`。
- 非阻塞检查：Hygiene（actionlint + shellcheck）、Coverage（cargo-llvm-cov）、
  MSRV check、deny 定时周扫（新披露的 RUSTSEC 公告 / 许可证问题即使无代码变更
  也会被扫出）。

## 10. 明确不适用的上游章节

以下工作区规则与本项目无关，勿在本仓库套用（原文见 `/mnt/codespace/AGENTS.md`）：

- §0.6 大文件下载纪律（sing-box / hf-mirror / 代理配额）；
- §1 节点访问表与 §8 node-2/3 部署、§8.1 malkuth 感知（本仓库走 Docker/GHCR）；
- §7.2 编译纪律与监听器优先（malkuth 监督体系）；
- §9 跨仓前端依赖、pnpm 家族、worktree 软链与 NFS 共享（本仓库纯 Rust、无
  前端包、不在 NFS 上开发）。
