+++
id = "br-37hz"
title = "Roles pass --ticket on every ticket spawn so the 10-minute hold applies (follow-up to 85hs)"
kind = "fix"
state = "built"
created_at = "2026-10-15T12:47:22.979Z"
updated_at = "2026-10-15T13:25:21.566451Z"
created_by = "agent:pm-1"
watchers = [
    "agent:pm-1",
    "agent:37hz",
]
branch = "bridle/37hz"
commit = "5e5c8f2b"
summary = "The manager role and skill now spawn workers and researchers with --ticket <id>, so the 10-minute settle hold (br-85hs) applies. Edited workflow/base/roles/manager.md and workflow/base/skills/manager/SKILL.md; CHANGELOG line added. The orchestrator's manager spawn is not for a ticket, so left alone. No daemon change; --ticket exists on agent spawn (cli.rs)."
ticket = "37hz"
+++

docs/tickets/open/roles-pass-ticket-on-every-ticket-spawn-so-the-10-minute-hol-37hz.md

## Thread

### note · agent:pm-1 · 2026-10-15T12:47:26.973Z
Brief (pm-1). Model: Haiku, docs only, small. Edit workflow/base/roles/manager.md (line ~36 and ~40, the worker/researcher/designer spawn lines) and workflow/base/roles/orchestrator.md (~129, manager spawn if it is for a ticket) so each ticket spawn reads `bridle agent spawn <role> --ticket <id> --name <short-name> --prompt "..."`. Check the other roles with `git grep -n "agent spawn" workflow/` and fix any ticket spawn. First confirm in crates/bridle/src (agent spawn args) that `--ticket` exists and what it does, so the text is accurate. Keep ASCII. If a role-prompt test or a duplicate copy under .bridle/ pins the text, update it. CHANGELOG entry. Acceptance: just check. Out of scope: daemon inference from --name; any daemon change (if --ticket turns out not to cover designer/planner spawns, stop and tell the manager). Touches the same role files as br-f2yd and br-qca7 (both pending): run after them if they land first.

### note · agent:37hz · 2026-10-15T13:25:16.331Z
done: manager role + skill spawn with --ticket <id>, CHANGELOG added; 5e5c8f2b. just check green (1708 passed) on the tree minus the CHANGELOG line, which I added mid-run (docs-only); main already merged.

### note · agent:manager-2 · 2026-10-15T13:25:21.566Z
built: 5e5c8f2b (branch bridle/37hz)
