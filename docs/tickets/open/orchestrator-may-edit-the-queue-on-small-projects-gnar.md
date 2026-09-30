---
id: gnar
title: The orchestrator may edit the queue on a small project with no product manager
opened: 2026-09-30
repos: [bridle]
changes: []
specs: []
needs: []
see: [w2hj]
---

## The ask

On meta-notes (run from the NUC, no product manager), the NUC's orchestrator tried
`bridle queue set` and got `forbidden: only the product manager or the human may edit the queue`,
so the human ran it. The human, verbatim (2026-09-30, relayed by the NUC's orchestrator, m-2735):

> so, we could start a pm here, but we hardly need one for this level of work. I ran the command
> to queue the work up, but this is a feature request: at the orch and humans discretion, when
> the amount of work is small, orch can schedule work; edit the queue. we don't need to spin up a
> pm on every small project, IMO.

## Today

`require_pm_or_human` in `crates/bridle-daemon/src/server.rs` is an allowlist: `human` and an
agent with role `product-manager`. `external:orchestrator` is neither.

## Related

[[small-projects-start-without-a-manager-w2hj|small projects start without a manager]]: the
same direction, fewer standing roles on small projects.
