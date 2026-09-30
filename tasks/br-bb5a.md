+++
id = "br-bb5a"
title = "CLI group split B: group commands into subcommands with hidden aliases for every old name (a67t)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T23:30:08.935Z"
updated_at = "2026-09-30T23:30:15.798219Z"
size = "M"
priority = "high"
+++

Ticket: docs/tickets/open/group-the-cli-into-subcommands-a67t.md ("A first grouping" table, "Decided"). Parent br-163f; depends on slice A (module split) being merged.

Goal: regroup the 54 top-level commands into the approved groups: daemon, agent, task, usage, orchestrator, workflow, hook (hidden), and the top-level leftovers (status, send, inbox, events, wait, tui, token, port, probe), so `bridle task claim`, `bridle agent spawn`, `bridle daemon serve` etc. work. Daily ones keep visible short top-level forms where the ticket suggests (status, send, inbox; agents, queue if cheap).

HARD REQUIREMENT: every old command line keeps working through hidden aliases (clap hide = true aliases / hidden top-level subcommands forwarding to the new ones) with identical flags and behaviour. Hooks, launchers, launchd plists, systemd units, scripts/ and wait-for-wake call the old names; self-upgrade installs this binary mid-flight, so a break would strand running agents. Add a test that runs EVERY old top-level name (from the ticket list of 54) with --help and asserts success, plus a few real invocations (e.g. `bridle statusline`, `bridle stop-check`, `bridle orchestrator note-session` argument shapes) through the old path. Also keep `bridle serve` exactly as launchd/systemd units spell it.

Also: update every reference to the new names in the same change (role prompts under workflow/ and .bridle/, skills, scripts/, docs/design/cli.md, hook settings generators); old names may stay where a running old client would still use them, but the docs describe the new. CHANGELOG entry.
Acceptance: just check passes; alias test above; `bridle --help` shows the grouped tree. Must merge and be self-upgraded before Thu 2026-10-01 10:00 ET or it waits until Sat 2026-10-03; if the manager sees it will miss that, tell pm-1 and do not land it late.
Model: Sonnet. Out of scope: completions (br-146a follows), dropping the aliases (a later release), any behaviour change.

## Thread

### note · agent:pm-1 · 2026-09-30T23:30:08.936Z
priority: normal -> high
