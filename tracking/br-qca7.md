+++
id = "br-qca7"
title = "aide prompt: say the orchestrator is <machine>-home, check its waiter there, and listen only on your own daemon"
kind = "bug"
state = "integrated"
created_at = "2026-10-15T01:10:32.118Z"
updated_at = "2026-10-15T13:08:48.633770Z"
created_by = "external:aide@remus"
watchers = [
    "external:aide@remus",
    "agent:qca7",
]
branch = "bridle/qca7"
commit = "1637a592aab49bdabb834fac9d0423a963bac6bc"
summary = "aide.md: says the one orchestrator is in <machine>-home, relay check uses bridle status --json --project <machine>-home, aide listens only on its own daemon (replies auto-forwarded). Docs only; CHANGELOG line added."
ticket = "qca7"
+++

The aide role prompt (workflow/base/roles/aide.md) doesn't say there is one orchestrator per machine. Filed by the data-contracts aide on remus at the human's request (2026-10-14 ~9:05 PM ET).

What happened: primed in data-contracts, the aide checked waiter_open on the data-contracts daemon (false) and told the human no orchestrator was watching. The orchestrator lives on remus-home. After sending to external:orchestrator --project remus-home it replied, and the reply arrived on the aide's own daemon.

Wanted in aide.md:
1. One orchestrator per machine: the <machine>-home project (e.g. remus-home). Today only a passing mention in the relay bullet.
2. "Relaying to the orchestrator" checks waiter_open on that daemon: bridle status --json --project <machine>-home.
3. The aide listens only on its own project's daemon; replies from the orchestrator and other daemons are forwarded there automatically (mail is auto-forwarded).

## Thread

### note · external:aide@remus · 2026-10-15T01:10:32.119Z
submitted by external:aide@remus

### note · agent:pm-1 · 2026-10-15T01:10:42.276Z
Brief (Haiku, docs only). Edit workflow/base/roles/aide.md only: the three points in the ask, in the "Relaying to the orchestrator" section and the relay bullet. Keep ASCII, short. If the project's workflow copy of the role must be kept in step (check for a duplicate under .bridle/ or a role-prompt test), update it too. Acceptance: just check passes. Out of scope: other roles (br-f2yd edits the roles for ticket submit --project; if it lands first, rebase on it, they may touch aide.md too).

### note · agent:qca7 · 2026-10-15T13:03:42.310Z
done: aide.md says orchestrator is <machine>-home, relay checks waiter there, aide listens only on own daemon; CHANGELOG added; just check exit 0, 1708 tests; 26f422b0

### note · agent:manager-2 · 2026-10-15T13:03:49.416Z
built: 26f422b0 (branch bridle/qca7)

### note · agent:manager-2 · 2026-10-15T13:08:36.024Z
integrated: 1637a592aab49bdabb834fac9d0423a963bac6bc (branch bridle/qca7)

### note · agent:manager-2 · 2026-10-15T13:08:48.633Z
cleanup: removed agent qca7, branch bridle/qca7
