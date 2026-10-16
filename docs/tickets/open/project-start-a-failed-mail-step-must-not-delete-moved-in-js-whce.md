---
id: br-whce
title: "project start: a failed mail step must not delete moved-in.json (the old machine's [mail] and mail.key copy is lost)"
kind: fix
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## The ask

Regression found in br-btpy by the orchestrator. In `carry_in` (crates/bridle/src/project_start.rs) a failed `mail_step` only prints, and moved-in.json is deleted anyway, so the [mail] and mail.key copy from the old machine is lost for good.

Fix: keep moved-in.json when the mail step fails (delete it only after the mail step succeeded), so the next `project start` retries. Test: a failing mail step leaves moved-in.json; a success removes it. Acceptance: just check. Model: Haiku or Sonnet. Out of scope: other project_start changes.
