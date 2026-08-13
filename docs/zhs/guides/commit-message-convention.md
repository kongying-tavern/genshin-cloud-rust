# 提交规范（gitmoji）

> [← 返回索引](../README.md) · 钩子由 `celestia-devtools` 提供

本项目遵循 celestia-island 组织的 [gitmoji](https://gitmoji.dev) 提交规范。
`just hooks`（底层 `celestia-devtools hook install --force`）会在 `.git/hooks/`
安装 commit-msg 钩子，在每次 `git commit` 时强制校验 subject 行。

## 规则

- **必须以 gitmoji 开头**（emoji 直接写在 subject 最前面）。
- **必须为英文**，不得出现中文 subject。
- **首字母大写**（gitmoji 之后的第一个英文字母）。
- **以句号 `.` 结尾**。
- **不得使用 Conventional Commits 前缀**：禁止 `feat:`、`fix:`、`chore:` 等。
- master / 主分支拒绝直接合并提交，仅接受 squash 合并。

示例：

```text
✨ Add area list endpoint.
🐛 Fix marker soft-delete filter.
⬆️ Bump sea-orm to 1.1.
```

## 常用 gitmoji

| Emoji | 含义 |
| --- | --- |
| ✨ | 新功能（feature） |
| 🐛 | 修复 bug |
| 📝 | 文档 |
| ♻️ | 重构 |
| ⬆️ | 升级依赖 |
| 🔧 | 配置 / 构建脚本 |
| ✅ | 测试 |
| 🚧 | 施工中（WIP） |
| 🎨 | 代码格式 / 结构调整 |
| 🔥 | 移除无用代码 |
| 🗑️ | 删除废弃文件 |

完整列表见 [gitmoji.dev](https://gitmoji.dev)。

## 安装与跳过

```bash
just hooks                                  # 安装 / 重装 commit-msg 钩子
CELESTIA_COMMIT_MSG_SKIP=1 git commit -m "..."  # 单次跳过校验（仅紧急情况）
just commit-msg-lint <file>                 # 手动校验某个 commit-msg 文件
```

> `CELESTIA_COMMIT_MSG_SKIP=1` 仅跳过本次校验，不应作为常规手段。规范的
> gitmoji subject 才是正确做法。主分支强制 squash 合并，因此历史中每个提交
> 都应是一个完整、自洽的 gitmoji 变更。
