---
id: fne2
title: "bridle session advisor and bridle advisor start: near-identical commands that do different things"
kind: arch-revision
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [9z2d, ervd, a67t, ukpm]
tasks: [br-fne2]
---

## The ask

The human, 2026-10-05 ~9:00 PM ET (verbatim): "Also, this looks like a big design problem to have session advisor start and advisor start. That's a huge, huge design problem. We've got two commands with almost the same syntax that do completely different things. File a ticket for that."

## The two commands today

- `bridle session advisor [name] [claude args]` **is** the advisor: it registers the session, signs the name and execs `claude` in the current terminal. Meant for a human at a terminal. Without a TTY it runs one print-mode turn and exits 0.
- `bridle advisor start <name> [--brief]` **launches** an advisor elsewhere: optionally mails a brief, then `tmux split-window` beside the orchestrator's pane (or a new window) running `bridle session advisor <name>`. Meant for the orchestrator (crates/bridle/src/advisor.rs, ticket ervd).

Same noun, near-identical spelling, different jobs (be the session vs start one elsewhere). The orchestrator's prime names both, one line apart. Incident 9z2d is what that costs.

## To decide

- One verb per action, so the wrong one can't be reached by habit. Options: fold `advisor start` into `session advisor --pane` (or `--split`); or rename `session advisor` to something that says "run here" and keep one `advisor` command for launching; or one command that splits when called from a non-TTY inside tmux.
- The same for `session aide`, `session orchestrator` and any other `session <role>`. Does each need a "start it elsewhere" form?
- `session *` refuses without a TTY (exit non-zero, "needs a terminal; to start one in a tmux pane use ...").
- `--help` for each names the other and says when to use it.
- Fits ticket a67t (group the CLI into subcommands).

## The human's direction (2026-10-05 ~9:25 PM ET, verbatim)

"Some of the complexity here will get better if we get away from tmux, but this needs to be merged into one command. What needs to be done needs to be differentiated somehow. We have one command that starts an agent, and then an option to tell it to use a tmux pane for that, like a new pane or a different pane. It could auto-detect that this is being run from a session, potentially. I'm not sure, but when the user types this into a shell, it should just start right there. That's the normal behavior, and the exceptional behavior is if it's being run from an agent."

"I'm thinking this needs some configuration. The tmux splitting works, but it's very hacky. What I really want that to do is, when you start an agent in a tmux pane, it should come up and tag the pane with all of its information: project name, project role, and name. I think any agent that's not globally unique can have a name. Orchestrator does not get a name, but advisors and aids do. I don't see any reason we can't have multiple aids. I don't think it's a problem, but we could also restrict it just to advisors for now if we want. I'm fine with that too."

"We need to unify the CLI and the behavior of all of these things. If I start the track web aid and it's running in some pane somewhere, we know we can restart it with handoffs. We need to take this one step further: if it goes down or is shut off with an exit command, it starts in the last pane it was running in by default."

"We need to get really clear on the design and think through all of the different steps of the design and what should be done with it. I'd actually be happy having an agent do this and then write up a ticket. I don't want a design document. I want a ticket. The ticket is, for now, the design document."

So: one command to start any interactive role (orchestrator, aide, advisor). It runs in place when a human types it in a shell; starting it elsewhere (new pane, a given pane) is an option, possibly auto-detected when an agent runs it. Panes are tagged with project, role and name. Named roles (advisors, maybe aides) vs the unnamed orchestrator. Restart in the last pane after a crash or exit. Configurable. The design goes in this ticket, written by the designer role (ticket ukpm), not a design doc.
