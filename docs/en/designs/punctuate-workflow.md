# Punctuate Workflow — Crowd-Sourced Submission, Audit, and Promotion

> Domain: `punctuate` (打点). Related code: `functions/api/punctuate.rs`,
> `functions/api/punctuate_audit.rs`, `database/models/marker/marker_punctuate.rs`.
> Java counterpart: `PunctuateService` (`stage` / `commit`) and
> `PunctuateAuditService` (`passPunctuate` / `rejectPunctuate`).

## Why this workflow exists

The 空荧酒馆 Genshin Map is a **crowd-sourced** map: the overwhelming majority of
its tens of thousands of points of interest — Anemoculi (风神瞳), Geoculi
(岩神瞳), chests (宝箱), specialty farming nodes, puzzle hints — were not entered
by a paid team. They were contributed by ordinary players tapping a location on
the map, attaching an icon, and submitting it.

That contribution model is powerful but dangerous. Without a gate, a single
malicious or careless user could:

- Drop a hundred bogus markers in the middle of the Mondstadt lake.
- Move every chest in Liyue two pixels east.
- Delete a region's worth of submissions out of spite.

So a contribution is **never** written directly to the live `marker` table. It
lands in a separate staging table — `marker_punctuate` — and stays there until a
trusted editor reviews it. Only after an editor approves does the submission
become a real, queryable marker. This is the punctuate (打点) workflow, and the
entire data-integrity story of the map rests on it.

## The two tables

```text
marker            ← live, served to every client, queried by *_doc exports
marker_punctuate  ← staging; only the contributor and auditors see it
```

`marker_punctuate` intentionally mirrors most columns of `marker`
(`marker_title`, `position`, `content`, `picture`, `video_path`,
`refresh_time`, `hidden_flag`, `extra`) but adds audit metadata on top:

| Column | Purpose |
| --- | --- |
| `punctuate_id` | The client-side submission identifier (one contributor can stage many). |
| `original_marker_id` | Only meaningful for `Modified` / `Deleted` submissions — the live marker being edited or removed. |
| `author` | The submitting user (becomes the marker's `creator_id` on promotion). |
| `status` | The state-machine column. See below. |
| `method_type` | Which kind of edit this submission represents. See below. |
| `audit_remark` | Free text from the auditor, set on rejection. |

Both tables carry the workspace-wide `version` (optimistic lock), `del_flag`
(soft delete), and `creator_id` / `updater_id` / `create_time` / `update_time`
pair. See [the architecture guide](../guides/architecture.md) for the
`SafeEntityTrait` that enforces these.

## The status state machine

`status` is the `MarkerPunctuateStatus` enum
(`packages/utils/src/types/marker.rs`). The Rust names map to the Java enum
names as follows:

| Rust state | Numeric value | Java name | Chinese | Meaning |
| --- | --- | --- | --- | --- |
| `Pending`   | 0 | `STAGE`  | 暂存 | Staged by the contributor, not yet handed to an editor. The contributor can still edit or discard it. |
| `Reviewing` | 1 | `COMMIT` | 审核中 | Committed to the audit queue. An editor will pick it up from `do_get_page_all`. |
| `Rejected`  | 2 | `REJECT` | 不通过 | An editor turned it down, with a written reason in `audit_remark`. The contributor may revise and re-commit. |

The contributor-side transitions are in `functions/api/punctuate.rs`:

```text
                 do_submit(status=Pending)         do_submit(status=Reviewing)
   (nothing) ─────────────────────────► Pending ─────────────────────────► Reviewing
                                             ▲                                    │
                                             │  do_submit(status=Reviewing)         │
                                             │  (re-commit after revision)          │
                                             └───────────────────────────────────── │
                                                                               │
                                          (auditor acts — see below)             │
                                                                               ▼
                                                                            Rejected
```

Key rules enforced by `do_submit`:

- **Staging** (`status = Pending`): creates a new `marker_punctuate` row, or — if

one already exists for the same `punctuate_id` in `Pending` or `Rejected`
state — overwrites it in place. This makes staging idempotent: the client can
keep autosaving without growing the table.

- **Committing** (`status = Reviewing`): only legal from `Pending` or `Rejected`.

The matching row's `status` is flipped to `Reviewing`. There is no direct path
to commit from nothing — the contributor must stage first.

- **Direct reject**: `do_submit(status = Rejected)` is explicitly rejected with

"不能直接将状态设为不通过；需通过审核驳回流程". Rejection is an auditor-only action.

`do_update` lets the contributor revise a `Pending` or `Rejected` submission but
refuses to touch anything already in `Reviewing` (the editor may be looking at
it right now).

## The three method types

`method_type` is the `MarkerPunctuateMethodType` enum. A submission is not just
"add a marker" — it can also propose an edit or a deletion of an existing live
marker.

| Rust variant | Value | Java name | Chinese | `original_marker_id` |
| --- | --- | --- | --- | --- |
| `Added`    | 1 | `ADDED`    | 新增 | ignored |
| `Modified` | 2 | `MODIFIED` | 修改 | required — the live marker to update |
| `Deleted`  | 3 | `DELETED`  | 删除 | required — the live marker to soft-delete |

This is why `original_marker_id` exists on `marker_punctuate`: it carries the
target of a `Modified` or `Deleted` submission and is meaningless for `Added`.

## Promotion: `do_pass` (audit approval)

`functions/api/punctuate_audit.rs::do_pass` is the moment a submission becomes
real data. It loads the `marker_punctuate` row (only if it is in `Reviewing`),
then branches on `method_type`. This is the direct port of Java's
`PunctuateAuditService.passPunctuate`.

```text
  Added    ──► INSERT into marker (creator ← author) ──► hard-delete punctuate row
  Modified ──► UPDATE marker SET ... WHERE id = original_marker_id
                                                  ──► hard-delete punctuate row
  Deleted  ──► delete_safety(original_marker) (soft-delete: del_flag = true)
                                                  ──► hard-delete punctuate row
```

Two details are easy to miss:

1. **The punctuate row is hard-deleted on success.** Unlike live entities,

`marker_punctuate` rows are *not* soft-deleted here. They have served their
purpose once promoted; keeping them would only bloat the audit queue. (The
`before_delete` guard in `SafeEntityTrait` is bypassed because the Rust code
uses the raw `Entity::delete_by_id`, not `delete_safety`.)

1. **`Modified` overwrites selectively.** `marker_title`, `position`, `content`,

`refresh_time`, and `hidden_flag` are always overwritten from the submission;
`picture` and `video_path` only if the submitter provided them, so an editor
can approve a content fix without wiping the contributor's earlier photo.

On all three paths the response is `{ "id": <marker id> }` — the new id for
`Added`, the original id for `Modified` / `Deleted`.

## Rejection: `do_reject`

`do_reject` flips `status` to `Rejected` and stores the auditor's free-text
`audit_remark`. The row stays in `marker_punctuate` so the contributor can read
the remark, revise via `do_update` (allowed from `Rejected`), and re-commit via
`do_submit(status = Reviewing)`. This loop — commit, reject, revise, re-commit —
is the normal path for a submission that is *almost* right.

## Contributor scoring

Approved submissions are not thrown away after promotion. The history table
records every `do_pass` as a `HistoryEditType` event, and the score pipeline in
`functions/api/score.rs::do_generate_score` (porting Java
`ScoreGenerateService.generateScorePunctuate`) periodically:

1. Clears the `score_stat` rows for the requested scope/span/time window.
1. Scans `history` for punctuate-related edits in that window.
1. Buckets them by `creator_id` (= the original submitter).
1. Writes one aggregated `score_stat` row per contributor.

Those rows are what feed the contributor leaderboards and the trust signals that
eventually gate who can submit directly vs. who needs heavier review. The Rust
port is currently a simplified edit-count version of the Java field-level diff
(`ScoreDataPunctuateVo`); the aggregation shape and the `score_stat` table are
already in place so the diff can be tightened later without a schema change.

## Why the split matters in practice

The split between `marker_punctuate` and `marker` is the single most important
data-integrity decision in the project. Because every crowd-sourced edit flows
through it:

- **The `*_doc` export pipeline** (see

[BinaryMD5 archive export](./binarymd5-archive-export.md)) can ignore punctuate
entirely — it only reads `marker`, so unreviewed submissions never reach the
cold-start blob.

- **The `find_safety()` read path** serves only promoted markers; the two flags

in [Hidden and special flags](./hidden-and-special-flags.md) then gate which
promoted markers each user sees.

- **Auditors get a single queue** (`do_get_page_all` filters

`status = Reviewing`), not a diff against the live table.

If you ever find yourself wanting to write directly to `marker` to "just fix one
thing quickly," you are bypassing the workflow that keeps the map trustworthy.
Go through `do_pass` instead.
