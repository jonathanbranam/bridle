---
id: ubjd
title: A CHANGELOG line for every landed task, written on the branch, and one section per kind under Unreleased
kind: chore
opened: 2026-10-09
filed_by: external:aide
repos: [bridle]
changes: []
specs: []
needs: []
see: [rcvb, 3ndf]
tasks: [br-ubjd]
---

## The ask

The human, verbatim (2026-10-08 ~8:45 PM ET):

> We should have a CHANGELOG written for every merged change - either worker commits and manager merges (and resolves inevitable conflicts) or the manager writes it and commits after the merge. I'd prefer it on the branch though. Commits are squashed right?

## Today (checked by the aide)
- `CHANGELOG.md` exists (Keep a Changelog format, "## Unreleased" on top).
- Entries are written on the worker's branch. The rule is `docs-current.md`: "add a line for user-facing or product changes". `manager.md` (~line 71) has the manager add the line in the worker's branch when it's missing.
- `bridle task land` squash-merges each task into one commit, so the entry lands inside that commit.
- **Gaps:** of the 22 task landings on main in the 36 hours to 10-08 ~8:40 PM ET, 6 had no CHANGELOG line: br-7h8e, br-5p3z (flaky-test fixes), br-at2j (WSL2 guide), br-4yc8, br-npj2 (research), br-xv2n (role text). All fell outside "user-facing", but the human wants **every** merged change listed.
- **The file's structure has drifted:** "Unreleased" has repeated `### Added` / `### Fixed` blocks, because each branch adds its own on top.

## The ask
1. **Every landed task gets a CHANGELOG line, written on the branch** (the human's preference). Internal changes such as tests, docs and research go under their own heading (e.g. `### Internal`), so the user-facing part stays readable.
2. **`bridle task land` refuses**, or the manager adds the line before landing, when the branch has no entry. The manager resolves the conflicts that come up in CHANGELOG.md. A merge driver or a fixed insertion point could make those conflicts rare.
3. **Keep one `### Added` / `### Fixed` / ... section under "Unreleased",** not one per landing.

Related: rcvb (daily "what happened" report), which could read the CHANGELOG.
