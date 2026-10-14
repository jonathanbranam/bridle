+++
id = "br-j4bk"
title = "integration/target grows without limit (833K files, 152 GB): prune it so syspolicyd stops stalling every land, check and self-upgrade"
kind = "fix"
state = "built"
created_at = "2026-10-14T14:44:08.112Z"
updated_at = "2026-10-14T15:22:53.461388Z"
created_by = "external:orchestrator"
watchers = [
    "external:orchestrator",
    "agent:j4bk",
]
branch = "bridle/j4bk"
commit = "094faeb8"
summary = "After each successful warm build, if target/debug/deps holds over 100,000 entries, warm_build::prune_stale deletes entries of debug/{deps,build,incremental} older than that build's start (reused-but-unchanged artifacts are rebuilt once next time). worktree::warm_target now skips the clone with a warning when the source's deps exceeds the same threshold. Tests: prune keeps current and removes stale; warm_target skips a bloated source. Docs: roles-and-config.md, CHANGELOG. Caveat: the check ran while main (docs-only changes) was being merged."
ticket = "j4bk"
+++

docs/tickets/open/integration-target-grows-without-limit-833k-files-152-gb-pru-j4bk.md

## Thread

### note · agent:j4bk · 2026-10-14T15:08:53.579Z
j4bk: implemented and committed on bridle/j4bk (main merged, 094faeb8); just check running in background, waiting on .bridle/check.exit. Next: summary, then message manager-2 with sha and result.

### note · agent:j4bk · 2026-10-14T15:22:41.865Z
done: warm build prunes stale target artifacts past 100K deps entries; warm_target skips bloated source; just check exit 0, 1671 tests passed (check began just before the docs-only main merge); 094faeb8

### note · agent:manager-2 · 2026-10-14T15:22:44.199Z
built: 094faeb8 (branch bridle/j4bk)

### note · agent:manager-2 · 2026-10-14T15:22:46.993Z
Your branch commits a stray .bridle/check.log. Remove it (git rm), commit, and message me the new sha. No re-check needed.

### note · agent:j4bk · 2026-10-14T15:22:53.461Z
removed stray .bridle/check.log; new sha a4b3aaca
