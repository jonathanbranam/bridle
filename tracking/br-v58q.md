+++
id = "br-v58q"
title = "bridle usage --week and --by name: the account breakdown in the CLI (ktgd step 4)"
kind = "feature"
state = "claimed"
created_at = "2026-10-13T23:05:38.577Z"
updated_at = "2026-10-14T09:08:04.619373Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:v58q",
]
ticket = "v58q"
+++

docs/tickets/open/bridle-usage-week-and-by-name-the-account-breakdown-in-the-c-v58q.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:44.385Z
Brief (pm-1). Model: Sonnet. Size: S. Blocked by br-pemk and br-zn6s. The ask is the spec. Extend `bridle usage` (crates/bridle/src; the existing `--by`, `--since`, `--history`, `--agent` flags from xxw9/xfy7) with `--week` (periods table: the week, days, five-hour windows, peaks, hits) and `--by name` / session from the new daemon routes. Table output plus `--json`. CLI test against the fake daemon. Docs: cli.md, CHANGELOG.md. Out of scope: gateway, UI.
