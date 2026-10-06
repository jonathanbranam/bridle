---
id: fne2
title: "bridle session advisor and bridle advisor start: near-identical commands that do different things"
kind: arch-revision
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [9z2d, ervd, a67t]
tasks: []
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
