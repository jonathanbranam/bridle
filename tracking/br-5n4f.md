+++
id = "br-5n4f"
title = "projects start / take-over don't re-create a project's tokens after a fresh database, and say nothing (split from wfcx)"
kind = "fix"
state = "built"
created_at = "2026-10-14T03:53:28.487Z"
updated_at = "2026-10-14T05:36:39.328096Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:5n4f",
]
branch = "bridle/5n4f"
commit = "a7ce287d1855374724d641e65ab17b3f0da6c270"
summary = "`bridle project start` now runs `token pair --projects P` on every run (project_start.rs carry_in), not only when take-over left moved-in.json, so a fresh database gets its role tokens back; pair already prints minted/kept per machine and names an unreachable one. If pairing is incomplete, start fails with the `bridle token pair --projects P` command to finish. Schedules/mail are carried only when the note exists. ssh timeouts (10 s connect, 60 s total) added via shared project_sync::run_timed(_raw), used by token_pair Exec and preflight_remote Ssh. Tests: existing token_pair fresh-mint and unreachable tests cover the logic; new run_timed timeout test; token_pair_test argv updated. Caveat: carry_in itself (needs a live daemon) has no unit test; pair output is buffered and shown after it finishes. Docs: cli.md, cli.rs help, CHANGELOG."
ticket = "5n4f"
+++

docs/tickets/open/projects-start-take-over-don-t-re-create-a-project-s-tokens-5n4f.md

## Thread

### note · agent:pm-1 · 2026-10-14T03:55:03.760Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body is the spec. Human needs this soon (notes/dotfiles-local/meta-notes-ui orchestrators were cut off by 401s after the rename). Start: `bridle projects start` and take-over code (git grep -n "re-creates" crates/bridle, and crates/bridle/src/token_pair.rs which holds the token pair code; xqwy changed it, read its landing 7fc14218 first). Make start/take-over mint missing role tokens on every run that finds them missing, print minted/kept per machine, and on an unreachable machine fail loudly naming it and printing the `bridle token pair` command to finish (wfcx added ConnectTimeout=10 and a 60 s limit to project_sync::ssh; token_pair.rs and preflight_remote.rs ssh calls still lack timeouts: add them here since this path uses them). Tests: fresh database -> start re-creates role tokens; fake unreachable machine -> named failure. Docs: cli.md, CHANGELOG.md. No migration. Out of scope: the `ticket comment --project` prefix bug.

### note · agent:5n4f · 2026-10-14T05:36:39.328Z
built: a7ce287d1855374724d641e65ab17b3f0da6c270 (branch bridle/5n4f)
