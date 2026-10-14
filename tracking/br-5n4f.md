+++
id = "br-5n4f"
title = "projects start / take-over don't re-create a project's tokens after a fresh database, and say nothing (split from wfcx)"
kind = "fix"
state = "claimed"
created_at = "2026-10-14T03:53:28.487Z"
updated_at = "2026-10-14T04:23:13.262536Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:5n4f",
]
ticket = "5n4f"
+++

docs/tickets/open/projects-start-take-over-don-t-re-create-a-project-s-tokens-5n4f.md

## Thread

### note · agent:pm-1 · 2026-10-14T03:55:03.760Z
Brief (pm-1). Model: Sonnet. Size: S-M. The ticket body is the spec. Human needs this soon (notes/dotfiles-local/meta-notes-ui orchestrators were cut off by 401s after the rename). Start: `bridle projects start` and take-over code (git grep -n "re-creates" crates/bridle, and crates/bridle/src/token_pair.rs which holds the token pair code; xqwy changed it, read its landing 7fc14218 first). Make start/take-over mint missing role tokens on every run that finds them missing, print minted/kept per machine, and on an unreachable machine fail loudly naming it and printing the `bridle token pair` command to finish (wfcx added ConnectTimeout=10 and a 60 s limit to project_sync::ssh; token_pair.rs and preflight_remote.rs ssh calls still lack timeouts: add them here since this path uses them). Tests: fresh database -> start re-creates role tokens; fake unreachable machine -> named failure. Docs: cli.md, CHANGELOG.md. No migration. Out of scope: the `ticket comment --project` prefix bug.
