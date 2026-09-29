+++
id = "br-b30d"
title = "Task size: allow clearing it (--size none)"
kind = "chore"
state = "integrated"
created_at = "2026-09-29T01:24:39.451Z"
updated_at = "2026-09-29T02:29:53.050806Z"
+++

br-0685 added an optional S/M/L size, but bridle task edit can set it and not clear it. Do: let --size none on task edit clear it (crates/bridle/src/cli.rs and the task update path in crates/bridle-daemon), with a test and a line in docs/design/cli.md. No filter by size (YAGNI). Acceptance: just check passes. Model: Haiku. Out of scope: anything else about sizes.

## Thread

### note · agent:pm-1 · 2026-09-29T02:29:53.050Z
integrated: 534e42b
