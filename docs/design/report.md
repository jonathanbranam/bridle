# The daily report

> **Status (checked 2026-10-08):** Built: `bridle report` (`crates/bridle/src/commands/report.rs`), slice 1 of ticket rcvb. Adding it to the mail digest is slice 2 and not built.

`bridle report [--since 24h] [--write]` answers "what happened" for one project. Output is
Markdown with five sections, in order, each printing `none` when empty:

1. Features built and delivered: feature tasks integrated in the window (id, title, commit).
2. Bugs: identified (created in the window) and fixed and delivered (integrated in the window), listed separately.
3. Pending or blocked: tasks now `pending`, `open`, `planned` or `claimed`, one line each with why (blocked by an unintegrated `blocks` edge, claimed by, or waiting).
4. Incidents: incident tasks created or integrated in the window, with state and the last thread note as the impact/fix line.
5. Anything else: tasks created, other-kind tasks integrated, and commits on main in the window (count and the first 20 subjects).

## Decisions

- **Mechanical, no LLM.** Built from tasks, edges and `git log`, so it costs no tokens and is the same every time.
- **One project per report**: the project the command runs in (`--project`). A machine-wide roll-up is not built (YAGNI).
- **Notable is everything listed above**; no ranking.
- **Window**: the previous 24 hours ending now; `--since` takes `90m`, `24h`, `2d`.
- **Times** are US Eastern, written bare, from a built-in US-rule offset (no timezone database); ASCII only.
- **"Integrated at"** is the time of the daemon's `integrated: <sha>` thread note; tasks have no such field, so a task without the note falls back to its last update. No wire change.
- **Daemon restarts/upgrades** are skipped: the API does not expose them as a list.
- **`--write`** saves `docs/reports/YYYY-MM-DD.md` (Eastern date), overwriting the same day's file and creating the folder; it never commits.
- **Commits** come from `git log main` in the current directory (else `HEAD`).
