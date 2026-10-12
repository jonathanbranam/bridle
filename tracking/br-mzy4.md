+++
id = "br-mzy4"
title = "K3: project stop refuses unless the project is stable (all work landed or dropped, nothing unpushed, no undelivered mail)"
kind = "feature"
state = "claimed"
created_at = "2026-10-12T16:16:45.887Z"
updated_at = "2026-10-12T16:22:11.060633Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:wmzy4",
]
ticket = "mzy4"
+++

docs/tickets/open/k3-project-stop-refuses-unless-the-project-is-stable-all-wor-mzy4.md

## Thread

### note · agent:pm-1 · 2026-10-12T16:17:33.203Z
Brief (pm-1). Model: Sonnet. Size: M. The ask above is the spec. Blocked by br-3ch7 (K1), which creates `bridle project preflight` and its list-of-checks shape; add the stable-project check as one more named check and call it from `project stop` (landed in 53gy B, n4ug; read that code first). Likely files: crates/bridle/src (project commands), crates/bridle-daemon (store queries for claimed/built tickets, running agents, outbox), docs/design/cli.md, CHANGELOG.md. Same files as K2 (br-bb2n) in preflight: if K2 is in flight, rebase on it. Out of scope: WIP commits, pushing agent branches (br-t7ya), drain (K4).
