---
id: br-madb
title: "bridle project setup stop points 2 and 3: clone from URL, root prompt, main/adopt question, aide token, next-step hints"
kind: feature
created: 2026-10-16
created_by: agent:pm-1
repos: [bridle]
changes: []
specs: []
blocked_by: [br-xtgf]
related: []
parent: br-dffz
resolved: 2026-10-16T10:02:59Z
---

## The ask

Slice 2 of epic br-dffz, after br-xtgf (stop point 1, built: driver, --check, register/state/service/tokens/mail steps in crates/bridle/src/project_setup.rs). Read br-xtgf's brief and br-dffz's design sections first.

Stop point 2: clone from a URL / `--remote`; the root prompt with the `<p>-work/<p>` default (always shown and confirmed, `--root` overrides, `-y` takes it); the main-or-adopt question when commits exist (default adopt, rule existing-projects; `--main`/`--adopt` answer up front); the aide token [Y/n]; no terminal and no -y: list the missing answers and exit 2. No `gh repo create`.
Stop point 3 (only if small): the "next: <command>" line the step commands print, computed from the same inspector.
Docs: cli.md section, docs/context/adding-a-project.md becomes "run bridle project setup", CHANGELOG. Verify in a throwaway workspace; never touch an existing project's main or dev. Acceptance: just check. Model: Sonnet. Blocked-by br-xtgf (same module). Out of scope: running setup for a project owned by another machine.
