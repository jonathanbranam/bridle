---
id: br-xgkf
title: "project setup gaps: failed init commit, declined aide token not remembered, next: hints beyond init"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
parent: br-dffz
---

## The ask

Follow-up to br-madb (landed; PdM verification 2026-10-16). Parent br-dffz.

1. Correctness: a failed init commit leaves files uncommitted, and the next run treats the init step as done. The init check must see uncommitted init files as unmet (or the apply must fail loudly and be retried).
2. A declined aide token ([Y/n] answered no) is not remembered, so every run asks again. Remember the decline in a way the check respects (no progress file per the design; use real state such as a marker in project config).
3. The "next: <command>" hints exist only on init. Add them to projects add, project start, token pair and mail install, computed from the same inspector.

Files: crates/bridle/src/project_setup.rs and the command modules. Acceptance: just check, tests for each. Docs: cli.md. Model: Sonnet. Out of scope: gh repo create.
