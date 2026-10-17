---
id: br-2fqg
title: "Worker role says run 'ticket built', but the daemon refuses it for workers (forbidden: agent lifecycle endpoints)"
kind: bug
created: 2026-10-17
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: []
---

## What happened

2026-10-17: workers w3stk (br-3stk, 12:51Z) and wz5kz (br-z5kz, 17:43Z) both ran `bridle ticket built <id> --branch ... --commit ...` as `workflow/base/roles/worker.md:31` tells them, and got `forbidden: worker agents cannot use agent lifecycle endpoints`. Each then posted a second note asking the manager to mark it built; the manager did it by hand.

## The ask

Make the two agree: either the daemon lets a worker mark its own claimed ticket built (it releases the worker, so it may touch the lifecycle endpoint the refusal guards), or the role drops the step and says the manager marks it built from the done note. Pick the one the permissions design (docs/design/agent-host/principals.md) intends.

Acceptance: a worker following worker.md finishes with no refused command; tests for whichever side changes.
