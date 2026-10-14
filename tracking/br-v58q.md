+++
id = "br-v58q"
title = "bridle usage --week and --by name: the account breakdown in the CLI (ktgd step 4)"
kind = "feature"
state = "integrated"
created_at = "2026-10-13T23:05:38.577Z"
updated_at = "2026-10-14T11:10:42.742695Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:v58q",
]
branch = "bridle/v58q"
commit = "b09d96d304a3df4f813abea709526cfaadb47e06"
summary = "bridle usage --week (latest week: days, five-hour windows, peaks, hits) and --by name|session (role/name/session cost tree with share-of-week estimates) added in crates/bridle/src/commands/usage.rs; both support --json. Test: crates/bridle/tests/usage_test.rs (fake daemon). cli.md and CHANGELOG updated. --week shows the latest week only; --since is passed to periods."
ticket = "v58q"
+++

docs/tickets/open/bridle-usage-week-and-by-name-the-account-breakdown-in-the-c-v58q.md

## Thread

### note · agent:pm-1 · 2026-10-13T23:06:44.385Z
Brief (pm-1). Model: Sonnet. Size: S. Blocked by br-pemk and br-zn6s. The ask is the spec. Extend `bridle usage` (crates/bridle/src; the existing `--by`, `--since`, `--history`, `--agent` flags from xxw9/xfy7) with `--week` (periods table: the week, days, five-hour windows, peaks, hits) and `--by name` / session from the new daemon routes. Table output plus `--json`. CLI test against the fake daemon. Docs: cli.md, CHANGELOG.md. Out of scope: gateway, UI.

### note · agent:v58q · 2026-10-14T09:32:21.174Z
done: bridle usage --week and --by name|session (+--json), test, cli.md, CHANGELOG; just check exit 0, 1668 tests run, 1668 passed (last full 1665); f942e1f2 (check ran on 1 commit before the merge of main, which only added a ticket file)

### note · agent:manager-2 · 2026-10-14T09:32:23.367Z
built: f942e1f2 (branch bridle/v58q)

### note · agent:manager-2 · 2026-10-14T11:10:42.742Z
integrated: b09d96d304a3df4f813abea709526cfaadb47e06 (branch bridle/v58q)
