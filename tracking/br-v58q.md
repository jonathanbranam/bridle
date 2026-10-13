+++
id = "br-v58q"
title = "bridle usage --week and --by name: the account breakdown in the CLI (ktgd step 4)"
kind = "feature"
state = "planned"
created_at = "2026-10-13T23:05:38.577Z"
updated_at = "2026-10-13T23:06:47.808548Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "v58q"
+++

docs/tickets/open/bridle-usage-week-and-by-name-the-account-breakdown-in-the-c-v58q.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:44.385Z
Brief (pm-1). Model: Sonnet. Size: S. Blocked by br-pemk and br-zn6s. The ask is the spec. Extend `bridle usage` (crates/bridle/src; the existing `--by`, `--since`, `--history`, `--agent` flags from xxw9/xfy7) with `--week` (periods table: the week, days, five-hour windows, peaks, hits) and `--by name` / session from the new daemon routes. Table output plus `--json`. CLI test against the fake daemon. Docs: cli.md, CHANGELOG.md. Out of scope: gateway, UI.
