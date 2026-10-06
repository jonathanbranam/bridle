---
id: a4pm
title: advisor start reuses the advisor's tagged pane before splitting
kind: feature
opened: 2026-10-06
repos: [meta-notes]
changes: []
specs: []
needs: []
see: []
tasks: []
---

## The ask


The human, 2026-10-05 (to the meta-notes orchestrator): "I was thinking that
we had a way to start a fresh agent in the same pane that it used last time
instead of always splitting. That's what I would prefer to happen. That
command should check and see if there is a tagged pane with the right
identifier, and it should start there. Only if that doesn't exist should it
split off of yours."

## Today

- `bridle advisor start <name>` (`crates/bridle/src/advisor.rs`,
  `tmux_args`) always runs `split-window` beside the pane tagged
  `@bridle=orchestrator`, or `new-window`. It never looks for the advisor's
  own pane.
- `bridle session advisor <name>` tags its pane `@bridle=advisor-<name>`
  (`crates/bridle/src/session.rs`, `crate::pane::tag_pane`).
- `bridle session restart advisor/<name>` already reuses the pane, with
  `tmux send-keys`, but only for a session that is still running.

## Wanted

`bridle advisor start <name>` first looks for a pane tagged with that
advisor's identifier. If there is one and nothing is running in it, it starts
the session there (`send-keys`, as `session restart` does). Only when there
is no such pane does it split beside the orchestrator (or open a window, per
`[tmux] advisor_pane`).

## Points for the plan

- **The pane has to survive the advisor.** `advisor start` passes the
  command to `split-window`, so the pane closes when claude exits and its tag
  goes with it. It needs to open a shell pane and type the command into it
  (or set `remain-on-exit` and use `respawn-pane`), so a quit advisor leaves a
  tagged pane behind to reuse.
- **Busy pane.** If the tagged pane still runs claude (the advisor is up),
  don't type into it: say it's running and point at `bridle session restart`.
- **The tag doesn't name the project.** `advisor-<name>` is the same for
  every project, so two projects' advisors with one name share a tag and
  `tag_pane` moves it between them. The tag (or the lookup) should include
  the project.
- The orchestrator's role text (`bridle advisor start` in
  `workflow/base/roles/orchestrator.md`) needs a line on the new behaviour.
