+++
id = "br-df68"
title = "Several named advisors: claude-advisor [name] (sfb3 follow-up)"
kind = "chore"
state = "integrated"
created_at = "2026-09-30T01:54:26.444Z"
updated_at = "2026-09-30T02:04:02.322794Z"
branch = "bridle/named-advisors"
commit = "a44361853f6de7e18bd532eeda2d8c8b7b706d47"
summary = "Implemented multiple named advisors and shorter session names. scripts/claude-advisor [name] allows running several advisors at once with session names 'advisor' (or 'advisor-<name>'); orchestrator session names now 'orch' (or 'orch-<suffix>'). All advisors share one identity, token, and inbox; each signs messages with its name. Updated workflow/base/roles/advisor.md and docs/questions/open/session-names-per-machine-sfb3.md with design and costs. Example commands: 'scripts/claude-advisor' gives session 'advisor'; 'scripts/claude-advisor research' gives session 'advisor-research'; with BRIDLE_SESSION_SUFFIX=nuc, 'scripts/claude-advisor' gives 'advisor-nuc' and 'scripts/claude-advisor research' gives 'advisor-research-nuc'. Similarly, 'scripts/claude-orchestrator' gives session 'orch', and 'scripts/claude-orchestrator' with BRIDLE_SESSION_SUFFIX=nuc gives 'orch-nuc'."
+++

GOAL: several advisors can run at once, named at start. 'scripts/claude-advisor [name]' gives the session name bridle-advisor-<name> (also used for --remote-control); with no name it keeps today's host-based default (bridle-advisor-<host>, BRIDLE_SESSION_SUFFIX still honoured). Keep it simple: one external:advisor identity, one token, one shared inbox. workflow/base/roles/advisor.md must say other advisors may be running and that each signs its messages with its name. Files: scripts/claude-advisor, workflow/base/roles/advisor.md, docs/questions/open/*sfb3*.md (record the decision), CHANGELOG.md. Acceptance: just check passes; show the resulting command line for both the named and default case in the summary. Model: Haiku. Out of scope: separate advisor identities or tokens, orchestrator script changes, NUC setup.

## Thread

### note · agent:manager-2 · 2026-09-30T01:59:56.935Z
Addition from the human (via orchestrator): shorter session names, no 'bridle-' prefix: orchestrator 'orch' (and 'orch-nuc' on the NUC), advisors 'advisor-<name>'. Reading: orchestrator session = 'orch' by default, 'orch-<suffix>' when BRIDLE_SESSION_SUFFIX is set (e.g. nuc); this replaces the host-based default from br-47ba (so no hostname in the default). Advisor session = 'advisor-<name>' (name argument; when none given, 'advisor'). Apply to both scripts, docs quoting the names, and the sfb3 ticket; say in your summary how you read it.

### note · agent:named-advisors · 2026-09-30T02:03:22.637Z
done: Multiple named advisors and shorter session names implemented; scripts/claude-advisor [name] gives 'advisor' or 'advisor-<name>' sessions, 'scripts/claude-orchestrator' gives 'orch', with BRIDLE_SESSION_SUFFIX override support. All advisors share identity/token/inbox and sign messages with their name. Updated advisor.md and sfb3 ticket with design. just check: 767 tests passed. Commit 3a9da2c9c214846ee7cee10c28138743fbdbbcf6

### note · agent:manager-2 · 2026-09-30T02:03:29.093Z
integrated: a44361853f6de7e18bd532eeda2d8c8b7b706d47 (branch bridle/named-advisors)

### note · agent:manager-2 · 2026-09-30T02:04:02.322Z
cleanup: removed agent named-advisors, branch bridle/named-advisors
