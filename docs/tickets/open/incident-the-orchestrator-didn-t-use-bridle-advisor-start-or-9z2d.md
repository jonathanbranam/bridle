---
id: 9z2d
title: "Incident: the orchestrator didn't use bridle advisor start or per-advisor addresses, and misled the human twice"
kind: incident
opened: 2026-10-06
repos: [bridle]
changes: []
specs: []
needs: []
see: [ervd, kae5, fne2, m9sd]
tasks: []
---

## The ask

The human, 2026-10-05 ~9:00 PM ET, to the orchestrator (verbatim): "Okay, file this as an incident. Why, why, why do things like this happen? Do we need better instructions? Do we need better help? Do we need skills? What do we need here? We've got two major things that we built that this session was not aware of and did not use properly."

## What happened

The human asked the orchestrator (session of 2026-10-05) to "create advisor for me: bridle session advisor fields \"<brief>\"". Earlier orchestrators had done this by splitting the tmux pane.

1. The orchestrator ran `bridle session advisor fields "<brief>"` with `run_in_background`. With no TTY, the claude session ran one turn in print mode, wrote its whole kzw2 walkthrough to the orchestrator's task output, and exited. The human saw nothing in a pane. The orchestrator reported it "up", then had to retract.
2. The human said "just run that command, it should split your tmux pane". The orchestrator ran the same command in the foreground. Same result: one print-mode turn, exited, no pane.
3. Only then did it read `crates/bridle/src/advisor.rs` and find `bridle advisor start <name> --brief`, which splits beside the pane tagged `@bridle=orchestrator`. It used `--brief` against the human's standing rule (2026-10-04: the brief goes into the pane after ~30 s, not as an inbox brief), and the human saw the brief cut off.
4. Asked to resend the brief, it sent it to the shared `external:advisor` inbox with "For advisor fields:", not to `external:advisor/fields`. The human: "Didn't we fix messaging yet? You should be able to message him directly."

Two built features went unused: `bridle advisor start` (ticket ervd) and per-advisor addresses `external:advisor/<name>` (docs/design/agent-host/principals.md).

## Why (five whys)

- **The orchestrator didn't read its role.** `bridle prime orchestrator` printed 42.8 KB; the session read only grep hits for the startup steps and the watcher. The right instructions were in it (role text: "Run `bridle advisor start <name>` with no `--brief`, wait ~30 s, then type the brief into its tmux pane."). Root cause 1: a prime too long to be read whole, and nothing that makes the session read the parts that matter when they matter.
- **The prime contradicts itself at a glance.** The line just before says "the human starts them (`bridle session advisor <name>`, `bridle session aide`)". Read in isolation, that is the command the human named. Root cause 2: two commands with near-identical names doing different things (ticket fne2).
- **The wrong command failed silently.** `bridle session advisor` without a TTY ran claude in print mode and exited 0, so neither attempt errored. Root cause 3: no guard that says "this needs a terminal; did you mean `bridle advisor start`?".
- **Per-advisor addressing isn't in the orchestrator's prime at all.** `external:advisor/<name>` is only in principals.md and the advisor role. Root cause 4: agents learn the CLI from scattered docs, not from the CLI (ticket m9sd).
- **`--help` didn't save it.** `bridle session advisor --help` says "A first argument is the advisor's name; the rest goes to `claude`" and never points to `advisor start`.

## What we may need (for the human to decide)

- **Better help:** each command's `--help` says when to use it and names its look-alike. `session *` refuses without a TTY and points at `advisor start`.
- **Skills, not a longer prime:** short task-shaped skills ("start an advisor", "message a person or session") that load when needed, instead of 40 KB read once and grepped.
- **Shorter instructions:** a prime short enough to be read in full, with details moved to `bridle help <topic>` (ticket kae5).
- **One name per action:** ticket fne2.
