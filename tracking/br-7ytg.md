+++
id = "br-7ytg"
title = "The machine's home project is the orchestrator's only project: one orchestrator, there, and nowhere else"
kind = "feature"
state = "open"
created_at = "2026-10-18T13:26:42.072Z"
updated_at = "2026-10-18T13:32:42.203765Z"
created_by = "external:aide"
watchers = ["external:aide"]
ticket = "7ytg"
+++

docs/tickets/open/the-machine-s-home-project-is-the-orchestrator-s-only-projec-7ytg.md

## Thread

### note · agent:pm-1 · 2026-10-18T13:32:38.732Z
Brief (pm-1). This ticket is SLICE 1 of 2 (warn-only phase); the refusals are slice 2 (filed separately, blocked-by this one), so no machine can lose its orchestrator when slice 1 self-upgrades everywhere. Model: Sonnet. Read the Decided and Rollout sections above.
Slice 1 builds (nothing refuses):
- Home resolution (item 2): the machine's home project defaults to <host>-home and can be overridden in the global config (~/.bridle/config.toml); one function everything uses (reuse br-8uxe's code if it already has it).
- E: `bridle doctor` (and a line in `bridle status`) reports the machine's home, the override if any, every project on the machine whose config has `[orchestrator]` enabled but is not the home, and where the orchestrator's waiter/session actually is.
- A (warn only): a daemon that is not the home and has `[orchestrator]` enabled logs a warning at start and shows it in doctor; it keeps behaving as today.
- C/D (warn only): `bridle session orchestrator`, `bridle up`, `bridle prime orchestrator` and the supervisor relaunch resolve the project from the home; given another project they print a warning naming the home and the exact command, and still proceed. A machine with no home prints how to create one (D), still proceeds on the cwd's project.
- B (warn only): a message to `external:orchestrator` on a non-home project is still accepted and forwarded to the home (br-dg5g), but the SENDER gets a warning naming the exact `--project <host>-home` command. Check cross-machine mail (outbox) still routes to the target machine's home.
- Senders: update roles and docs to use the home address: CLAUDE.md, workflow/base/roles/orchestrator.md, workflow/base/roles/advisor.md ("Messaging the orchestrator" still offers `--project bridle`), the advisor primer, skills and scripts that name `--project bridle` for the orchestrator.
- Self-upgrade onto this build changes no behaviour except the warnings; say so in docs/design/agent-host/orchestrator-supervision.md and CHANGELOG. No store/config migration is needed (warn-only reads existing config); say that in the CHANGELOG.
Tests: home default and override; doctor lists a non-home `[orchestrator]` project; each launcher warns but proceeds; message to a non-home orchestrator still delivered with the sender warning.
Acceptance: just check. Out of scope: any refusal (slice 2), the machine daemon (kuw2).

### note · external:advisor/product-manager · 2026-10-18T13:32:42.203Z
From advisor (product-manager): brief for pm-1's plan (the human approved the design, via aide m-16618; the Rollout section is a requirement). Plan it as ordered slices, each its own task, chained by blocked_by:

1. Warn-only, no refusals: one check (E: doctor/status reports home, override, any non-home `[orchestrator]`, where the waiter is); launchers resolve the home (C, item 2's global-config override) but only WARN on a non-home `--project`; a non-home daemon with `[orchestrator]` warns at start (A); a message to `external:orchestrator` on a non-home project is still forwarded, with a warning naming `--project <host>-home` (B prep). Roles, primers, skills and docs switch to the home address (CLAUDE.md, orchestrator.md, advisor.md "Messaging the orchestrator", aide.md). Safe to self-upgrade onto.
2. Per-machine check and human steps (not code): after slice 1 is live on each machine, the aide checks each machine's orchestrator runs in, waits on and is configured in its home. Dalek needs br-e7s5 (restart dalek-home) and a relaunch first. Every stop/restart/exit/config edit the human must do is filed as a `--for-human` to-do with exact commands, at the point it is needed. This gates slice 3.
3. Refusals (items 1, 3, 4, A, C, D; B as (a): refuse with the exact `--project <host>-home` command; cross-machine outbox mail still routes to the target's home). Lands only after slice 2 is confirmed on dalek, vesta and remus. Ship so a daemon self-upgrading onto it with a misconfigured orchestrator doesn't strand it: refuse new launches, never kill a running one.

Priority normal, after the Stages MVP children in the work order. Epic: orchestrator-per-machine (roadmap #3). Note for z485 (orchestrator as a background agent, design waiting on the human): it assumes this ticket's home-only rule; slice 3's refusals must allow the orchestrator agent in the home.
