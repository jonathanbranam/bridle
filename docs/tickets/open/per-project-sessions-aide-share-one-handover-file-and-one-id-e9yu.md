---
id: e9yu
title: Per-project sessions (aide) share one handover file and one identity across projects; key them by project
kind: bug
opened: 2026-10-05
repos: [bridle-ui]
changes: []
specs: []
needs: []
see: [gtzx, ma8e]
tasks: []
---

## The ask


The human, verbatim (2026-10-04 ~8:25 PM ET, to the bridle-ui aide; dictated, "iOS file" is "aide
file"):

> The iOS file is always shared between projects. That's just ridiculously poor planning. File is
> critical, but fix that, or send a note to the orchestrator.

## What happened

- At 200k tokens the daemon told bridle-ui's aide: "write a short note ... to
  /Users/jbranam/.bridle/handover/aide.md". bridle's aide had already written its own note there at
  ~7:45 PM ET (bridle, same machine). Following the instruction would have overwritten it. bridle-ui's
  aide wrote to `~/.bridle/handover/aide-bridle-ui.md` instead, which a restarted session won't look for.
- The path is `$BRIDLE_HOME/handover/<identity, / as ->.md`
  (`docs/design/agent-host/orchestrator-supervision.md`, lines 228 and 244). Every project's aide is
  identity `aide` (`external:aide`, one session per project, per `workflow/base/roles/aide.md`), so
  the path is the same machine-wide.
- The same identity-only key caused incident br-h3ar the same evening: `pkill -f "bridle agent wake
  external:aide"` matched every project's aide waiter.
- Anything else keyed by identity alone (handover, the session registry, `bridle session restart aide`)
  may collide the same way for per-project roles. Not checked.
