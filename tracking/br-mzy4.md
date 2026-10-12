+++
id = "br-mzy4"
title = "K3: project stop refuses unless the project is stable (all work landed or dropped, nothing unpushed, no undelivered mail)"
kind = "feature"
state = "integrated"
created_at = "2026-10-12T16:16:45.887Z"
updated_at = "2026-10-12T17:19:40.629016Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wmzy4",
]
branch = "bridle/wmzy4"
commit = "b4020b51e4cec76fc2bb4524b55e79fe796e1da7"
summary = 'New crates/bridle/src/stable.rs: the "stable project" check (claimed/built tickets, running agents, undelivered outbox mail via store::unstable_in_db; bridle/* and integrate/* branches ahead of integration; integration vs origin; dirty main checkout incl. untracked, minus --leave paths, shown as LEFT). Added to project preflight and run by project stop, which refuses naming each failure. Both gain --leave PATH. Caveat: stop skips the "integration branch on origin" test because stop pushes that branch itself (take_over_test a_failed_push relies on it); preflight keeps it. Docs: cli.md, CHANGELOG.'
ticket = "mzy4"
+++

docs/tickets/open/k3-project-stop-refuses-unless-the-project-is-stable-all-wor-mzy4.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:33.203Z
Brief (pm-1). Model: Sonnet. Size: M. The ask above is the spec. Blocked by br-3ch7 (K1), which creates `bridle project preflight` and its list-of-checks shape; add the stable-project check as one more named check and call it from `project stop` (landed in 53gy B, n4ug; read that code first). Likely files: crates/bridle/src (project commands), crates/bridle-daemon (store queries for claimed/built tickets, running agents, outbox), docs/design/cli.md, CHANGELOG.md. Same files as K2 (br-bb2n) in preflight: if K2 is in flight, rebase on it. Out of scope: WIP commits, pushing agent branches (br-t7ya), drain (K4).

### note · agent:wmzy4 · 2026-10-12T17:08:33.764Z
built: 0e8bee6c (branch bridle/wmzy4)

### note · agent:wmzy4 · 2026-10-12T17:08:33.823Z
done: stable-project check in preflight + enforced by project stop (--leave); just check exit 0, 1609 tests passed, main merged; 0e8bee6c. Caveat: stop skips the 'integration on origin' test since stop pushes it itself (take_over_test relies on it).

### note · agent:manager-2 · 2026-10-12T17:09:24.348Z
main moved (br-3v2m landed, touches project_start.rs). Merge main, resolve any conflicts, re-run just check, report exit status, test count and sha.

### note · agent:wmzy4 · 2026-10-12T17:17:33.553Z
main merged (only cli.md conflicted, resolved); just check exit 0, 1613 tests passed; d7d505f8

### note · agent:manager-2 · 2026-10-12T17:17:40.838Z
integrated: b4020b51e4cec76fc2bb4524b55e79fe796e1da7 (branch bridle/wmzy4)

### note · agent:manager-2 · 2026-10-12T17:18:07.922Z
cleanup: removed agent wmzy4, branch bridle/wmzy4

### note · external:advisor/product-manager · 2026-10-12T17:19:40.629Z
advisor (product-manager): verified on origin/main (b4020b51). The worker's caveat (stop skips the 'integration on origin' check because stop pushes it itself) is fine: stop's own push confirms origin. Resolving.
