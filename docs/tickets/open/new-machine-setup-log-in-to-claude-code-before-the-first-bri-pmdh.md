---
id: br-pmdh
title: "New-machine setup: log in to Claude Code before the first bridle serve"
kind: docs
created: 2026-10-14
created_by: external:orchestrator
repos: [bridle]
changes: []
specs: []
blocked_by: []
related: [br-myw9]
theme: multi-machine
---

## The ask

The human, 2026-10-14 (relayed by the orchestrator on remus, m-12839), verbatim:

> ah, ok, that makes sense then, we need to add the claude login instructions to come before bridle
> serve on a new machine.

The relay: "The quote above is the human's approval to file it and mark it ready."

## The change

In the new-machine setup steps, log in to Claude Code (`claude`, then `/login`, or `claude
login`) before the first `bridle serve` / service start, and say why: a daemon started before the
login reports "claude is not logged in in this daemon's session", and its agents crash until the
daemon is restarted after a login (br-myw9: remus needed two logins).

- `docs/context/add-a-machine.md`: the prerequisites (line ~23 names Claude Code) and the
  numbered steps around `gh auth login` (step 2) get a "log in to Claude Code" step ahead of
  serving any project.
- `docs/context/windows-wsl2-host.md`: the same, if its steps start a daemon.

## Acceptance

Both guides put the Claude login before any `bridle serve` or unit start, with the one-line why.
Docs only. Model: Haiku.
