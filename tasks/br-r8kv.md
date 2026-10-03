+++
id = "br-r8kv"
title = "Split the interactive roles: triage talks to the human about the system; the orchestrator only runs it; advisors only talk and research (r8kv)"
kind = "feature"
state = "integrated"
created_at = "2026-10-03T21:18:51.985Z"
updated_at = "2026-10-03T22:25:53.171904Z"
created_by = "external:advisor"
watchers = ["external:advisor"]
branch = "bridle/triage-role"
commit = "3c1e6bf2deb0f2457dd97bc86b8f0e611dadfc9f"
summary = "Added the triage role (workflow/base/roles/triage.md), 'bridle session triage' (signs external:triage via BRIDLE_AS=triage, pane tag triage, focus-locked like advisors, not registered as a daemon session) and 'bridle prime triage'. Orchestrator role no longer talks with the human or lists their to-dos: it messages external:triage and takes relayed approvals. Advisor role only talks/researches/files tickets; its launcher prompt no longer asks for a status check-in. Daemon: 'triage' is a known external principal name and focus lock covers triage panes. Docs: cli.md, principals.md, orchestrator-supervision.md, CHANGELOG. Caveat: daemon system notes and question wakes still go to human/orchestrator (later slice); users need a [triage] token (bridle token create triage)."
+++

original id: r8kv
Ticket: docs/tickets/open/seats-named-interactive-roles-splitting-the-advisor-retiring-r8kv.md, section "Decided: split the roles, start now" (the human, 2026-10-03, verbatim there; approved to start now).
Goal: a clear three-way split of the interactive roles.
(1) New role `triage` (working name; the human may rename before it lands): workflow/base/roles/triage.md. It talks to the human about the running system: at start-up and on wake it reads the human's to-dos (`bridle task list --claimed-by human`), the workforce's questions to the human (GET /v1/messages?to=human), `bridle status` and incidents; it lays out options with a recommendation, relays the human's answers and approvals (quoting them) to the orchestrator and agents, and files tickets for what the human raises about the system. It waits and wakes (`bridle agent wake external:triage`). It doesn't run the workforce. Principal `external:triage`; `bridle session triage` starts it (like the advisor launcher); token under [triage] in credentials.toml; `bridle prime triage`.
(2) Orchestrator (workflow/base/roles/orchestrator.md): stays an interactive session for now, but stops talking with the human. It reaches the human only by messages to `external:triage` (not the human's inbox, not interactive conversation); startup no longer tells the human their to-dos (triage does); the human's approvals arrive relayed by triage (or an advisor). Keep its supervision, incident log, critical-fix rights. Remove or move to triage: "list the human's open to-dos and tell the human first thing", relaying to the human, "taking a discussion offline", inbox-for-the-human rules.
(3) Advisor (workflow/base/roles/advisor.md, .bridle/roles/advisor.md): only talks with the human, researches, files tickets, sends and receives messages; still waits and wakes. Remove: checking `bridle status` at start-up, triaging the human's inbox and the workforce's questions, relaying answers to agents (unless the human asks in conversation), anything about running work. The advisor launcher's opening prompt (`bridle session advisor`) must stop asking for a status check-in.
Also: update docs that name who talks to the human (orchestrator-supervision.md, roles docs, cli.md for `bridle session triage`, the human's to-do docs), CHANGELOG. Daemon changes only if needed (e.g. the supervisor's "no wake command" notes and system notes for the human could go to triage; keep scope small: role text and launcher first). Acceptance: just check passes; `bridle session triage` starts a session with the triage prime. Model: Sonnet. Migration: projects get the new role through the vendored workflow; no schema change. Out of scope: renaming, seats (one inbox per seat), headless orchestrator (z485).

## Thread

### note · agent:triage-role · 2026-10-03T22:22:57.953Z
done: triage role + session + prime, orchestrator/advisor roles split, docs and tests; ce954be

### note · agent:manager-2 · 2026-10-03T22:23:02.804Z
Diff reads fine. main moved; merge it, run just check, message me sha and result.

### note · agent:triage-role · 2026-10-03T22:24:21.015Z
done: main merged again (includes br-59vt 9bd4587), just check green (1107 passed); ec0847e

### note · agent:manager-2 · 2026-10-03T22:24:25.708Z
integrated: 3c1e6bf2deb0f2457dd97bc86b8f0e611dadfc9f (branch bridle/triage-role)

### note · agent:manager-2 · 2026-10-03T22:25:53.171Z
cleanup: removed agent triage-role, branch bridle/triage-role
