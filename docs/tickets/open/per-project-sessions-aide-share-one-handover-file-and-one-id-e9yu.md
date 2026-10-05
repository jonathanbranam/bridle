---
id: e9yu
title: Per-project sessions (aide) share one handover file and one identity across projects; key them by project
kind: bug
opened: 2026-10-05
repos: [bridle]
changes: []
specs: []
needs: []
see: [gtzx, ma8e]
tasks: [br-e9yu]
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

## The human's clarification

The human, verbatim (2026-10-04 ~8:30 PM ET, to the bridle-ui aide):

> Sorry, that description. The handover file is what I'm referring to. It seems to have a shared
> name. That file should be based on the ID of the agent, so the named agents get a different
> file. I don't even know why it's a file. The orchestrator's handover is some kind of note in the
> system.

Fact: the orchestrator's handover is a record in the daemon's store, not a file: table
`handovers(seq, id, role, project, body, created_at, created_by)`, written with `bridle handover
write`, read by `bridle prime orchestrator`, and also pushed to the state branch as
`handovers/<id>.md` (`docs/design/agent-host/orchestrator-supervision.md`, section 7). `handover
write` is limited to the human and `external:orchestrator` today, so aides and advisors are told
to write a file under `~/.bridle/handover/` instead.

The human, verbatim (2026-10-04 ~8:40 PM ET, to the bridle-ui aide, on the plan for br-e9yu):

> Yeah, obviously, I didn't review the design for that. There should be a write command for an
> agent to write a handover, and it should be fully managed. Nobody should be ready to file. This
> should all be managed by the system, and the system then can ensure that every agent in every
> project with the proper name has the right handover and that there's no confusion about
> anything. If that's the way, please send that over for me.

("ready to file" is likely "writing to a file", dictated.)
