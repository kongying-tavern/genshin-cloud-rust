# 提交規範（gitmoji）

> [← 返回索引](../README.md) · 鉤子由
> [celestia-devtools](https://github.com/celestia-island/celestia-devtools) 提供
> （安裝：`pip install git+https://github.com/celestia-island/celestia-devtools.git`）

本項目遵循 celestia-island 組織的 [gitmoji](https://gitmoji.dev) 提交規範。
`just hooks`（底層 `celestia-devtools hook install --force`）會在 `.git/hooks/`
安裝 commit-msg 鉤子，在每次 `git commit` 時強制校驗 subject 行。

## 規則

- **必須以 gitmoji 開頭**（emoji 直接寫在 subject 最前面）。
- **必須爲英文**，不得出現中文 subject。
- **首字母大寫**（gitmoji 之後的第一個英文字母）。
- **以句號 `.` 結尾**。
- **不得使用 Conventional Commits 前綴**：禁止 `feat:`、`fix:`、`chore:` 等。
- master / 主分支拒絕 merge 提交，僅接受 squash 合併。

示例：

```text
✨ Add area list endpoint.
🐛 Fix marker soft-delete filter.
⬆️ Bump sea-orm to 1.1.
```

## 常用 gitmoji

| Emoji | 含義 |
| --- | --- |
| ✨ | 新功能（feature） |
| 🐛 | 修復 bug |
| 📝 | 文檔 |
| ♻️ | 重構 |
| ⬆️ | 升級依賴 |
| 🔧 | 配置 / 構建腳本 |
| ✅ | 測試 |
| 🚧 | 施工中（WIP） |
| 🎨 | 代碼格式 / 結構調整 |
| 🔥 | 移除無用代碼 |
| 🗑️ | 刪除廢棄文件 |

完整列表見 [gitmoji.dev](https://gitmoji.dev)。

## 安裝與跳過

```bash
just hooks                                  # 安裝 / 重裝 commit-msg 鉤子
CELESTIA_COMMIT_MSG_SKIP=1 git commit -m "..."  # 單次跳過校驗（僅緊急情況）
just commit-msg-lint <file>                 # 手動校驗某個 commit-msg 文件
```

> `CELESTIA_COMMIT_MSG_SKIP=1` 僅跳過本次校驗，不應作爲常規手段。規範的
> gitmoji subject 才是正確做法。主分支強制 squash 合併，因此歷史中每個提交
> 都應是一個完整、自洽的 gitmoji 變更。
