# Commit Message Convention

This project enforces the `celestia-devtools` gitmoji convention. A
`commit-msg` hook (installed by `just hooks`) validates every commit, and a
standalone CI workflow (`.github/workflows/commit-msg.yml`) re-checks the
same rules on push and on every PR. Commits that fail the check are rejected
locally and will fail CI.

## The rules

Every commit **subject line** (the first line) must satisfy all of:

1. **Start with a gitmoji.** The literal emoji comes first, e.g. `✨`, `🐛`.

No `:sparkles:`-style shortcodes — the actual emoji character.

1. **Be in English.** Subjects in Chinese or any other language are rejected.
1. **Be capitalized.** The first letter after the gitmoji (and a single

space) is uppercase.

1. **End with a period.** `.` at the end of the subject.
1. **No Conventional Commits prefixes.** Do **not** use `feat:`, `fix:`,

`chore:`, `refactor:` — the gitmoji already encodes the intent. A subject
like `feat: add area list` will be rejected.

The body (everything after the blank line separating it from the subject) is
free-form and may be any language; only the subject is linted.

### Examples

```text
✨ Add the area list endpoint.
🐛 Fix optimistic-lock version bump on soft delete.
📝 Document the SafeEntityTrait pattern.
♻️ Refactor the DB connection map into DatabaseConnectionMap.
⬆️ Bump redis to 1.x and axum-extra to 0.12.
```

Bad subjects (will be rejected):

```text
feat: add area list            ← Conventional Commits prefix, no gitmoji, no period
✨ add area list               ← not capitalized, no period
新增地区列表接口。               ← not English, no gitmoji
```

## Common gitmojis

| Gitmoji | Use for |
| --- | --- |
| ✨ | New feature (`feat`). |
| 🐛 | Bug fix (`fix`). |
| 📝 | Documentation. |
| ♻️ | Refactor (no behavior change). |
| ⬆️ | Dependency bump. |
| 🔧 | Configuration / tooling (`rust-toolchain.toml`, `justfile`, CI). |
| ✅ | Tests. |
| 🚧 | Work in progress (use sparingly; prefer to land a complete change). |
| 🎨 | Format / style (`cargo fmt`, `rustfmt.toml`). |

The full reference lives at [gitmoji.dev](https://gitmoji.dev). When in doubt,
match the intent: a new capability is `✨`, a fix to existing behavior is `🐛`,
and anything that only reshapes code is `♻️`.

## Installing and skipping the hook

```bash
just hooks        # install (or refresh) the commit-msg hook
```

Run `just hooks` once per fresh checkout. The hook is the vendored
`celestia-devtools` hook; reinstall it after `git clean -dfx` or when switching
machines. `celestia-devtools` itself comes from
[celestia-island/celestia-devtools](https://github.com/celestia-island/celestia-devtools)
— install it once with:

```bash
pip install git+https://github.com/celestia-island/celestia-devtools.git
```

To **skip the check for a single commit** (emergencies only), set the escape
hatch inline:

```bash
CELESTIA_COMMIT_MSG_SKIP=1 git commit -m "..."
```

Do not leave `CELESTIA_COMMIT_MSG_SKIP` exported in your shell — it would
silently disable the hook for every subsequent commit, and CI would still
catch the violation.

## Master branch policy

The `master` branch rejects direct merge commits: contributions land as a
single **squashed** commit so the history stays linear and every subject
satisfies the hook. The CI commit-message workflow lints the full commit
range of a PR, so squash-merging a clean PR is the safe path.
