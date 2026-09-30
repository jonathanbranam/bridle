+++
id = "br-df68"
title = "Several named advisors: claude-advisor [name] (sfb3 follow-up)"
kind = "chore"
state = "planned"
created_at = "2026-09-30T01:54:26.444Z"
updated_at = "2026-09-30T01:59:56.935041Z"
+++

GOAL: several advisors can run at once, named at start. 'scripts/claude-advisor [name]' gives the session name bridle-advisor-<name> (also used for --remote-control); with no name it keeps today's host-based default (bridle-advisor-<host>, BRIDLE_SESSION_SUFFIX still honoured). Keep it simple: one external:advisor identity, one token, one shared inbox. workflow/base/roles/advisor.md must say other advisors may be running and that each signs its messages with its name. Files: scripts/claude-advisor, workflow/base/roles/advisor.md, docs/questions/open/*sfb3*.md (record the decision), CHANGELOG.md. Acceptance: just check passes; show the resulting command line for both the named and default case in the summary. Model: Haiku. Out of scope: separate advisor identities or tokens, orchestrator script changes, NUC setup.

## Thread

### note · agent:manager-2 · 2026-09-30T01:59:56.935Z
Addition from the human (via orchestrator): shorter session names, no 'bridle-' prefix: orchestrator 'orch' (and 'orch-nuc' on the NUC), advisors 'advisor-<name>'. Reading: orchestrator session = 'orch' by default, 'orch-<suffix>' when BRIDLE_SESSION_SUFFIX is set (e.g. nuc); this replaces the host-based default from br-47ba (so no hostname in the default). Advisor session = 'advisor-<name>' (name argument; when none given, 'advisor'). Apply to both scripts, docs quoting the names, and the sfb3 ticket; say in your summary how you read it.
