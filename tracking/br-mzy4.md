+++
id = "br-mzy4"
title = "K3: project stop refuses unless the project is stable (all work landed or dropped, nothing unpushed, no undelivered mail)"
kind = "feature"
state = "built"
created_at = "2026-10-12T16:16:45.887Z"
updated_at = "2026-10-12T17:08:33.823996Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wmzy4",
]
branch = "bridle/wmzy4"
commit = "0e8bee6c"
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
