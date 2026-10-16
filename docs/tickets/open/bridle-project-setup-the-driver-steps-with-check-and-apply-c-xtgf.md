---
id: br-xtgf
title: "bridle project setup: the driver (steps with check and apply, --check, idempotent) over clone, init, register, state, service, tokens, mail"
kind: feature
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-btpy]
related: []
parent: br-dffz
---

## The ask

Slice 1 of epic br-dffz, option C (design approved by the human 2026-10-14 ~6:15 PM ET, hold lifted 2026-10-15 ~10:15 PM ET: "Fix as much as you can tonight"). The full design is in br-dffz ("Design options", "Design approved", "The four open design points, answered"); read it first.

Build `bridle project setup [URL] [--remote URL] [--root DIR] [--main|--adopt] [-y] [--check]`: a driver over the existing steps in a fixed order (clone, init, register in [projects] and sync, bridle/state + owner, daemon service, tokens, mail, aide token [Y/n]). Each step has check (read real state) and apply (call the existing command's code, never copy it). Run check on all, print the checklist, apply the unmet ones in order, stop at the first failure with the exact fix and "run the same command again". `--check` prints the checklist and changes nothing. No progress file; a step that cannot tell is "unknown" and the run exits non-zero. Workspace root: `--root`, else the clone's own place, else the proposed default `<base>/<p>-work/<p>` always shown and confirmed (or overridden) before anything is written; `-y` takes it. No `gh repo create`: the human provides the remote. Commits present: ask main or bridle-adopt (default adopt, rule existing-projects); `--main`/`--adopt` answer up front. No terminal and no -y: list the missing answers, exit 2.
