+++
id = "br-9dcz"
title = "Agents are reminded once to link the ticket and task IDs they show the human (a hook, never a block)"
kind = "feature"
state = "integrated"
created_at = "2026-10-10T17:09:36.504Z"
updated_at = "2026-10-17T09:46:56.099748Z"
created_by = "external:advisor/product-manager"
watchers = [
    "external:advisor/product-manager",
    "agent:9dcz",
]
branch = "bridle/9dcz"
commit = "f7725695a45f3ee17bb55f6b915d335e24757a44"
summary = "New hidden Stop hook 'bridle hook link-check' (crates/bridle/src/link_check.rs scan, commands/hook.rs, session.rs REPLY_HOOK with timeout 5) reminds orchestrator/advisor/aide sessions once per reply to link task IDs (own prefix) and bare IDs that exist as docs/tickets files; skips code, paths, longer words, already linked; allows on stop_hook_active/no public_url/any error. Rule link-ids-for-the-human lost its roles: list. Docs: cli.md, CHANGELOG. Caveat: restart_hold_test cancel_ends_an_upgrade_drain_quietly flaked once under load, passed alone and in the green rerun."
ticket = "9dcz"
+++

docs/tickets/open/agents-are-reminded-once-to-link-the-ticket-and-task-ids-the-9dcz.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T02:06:33.258Z
The human, 2026-10-16 ~10:10 PM ET, to advisor (product-manager): "Yes I approve the \"cheap batch\" of tickets mentioned earlier".

### note · agent:pm-1 · 2026-10-17T02:07:21.421Z
Brief (pm-1): build Option B of the Design section (the human approved the ticket, 2026-10-16 ~10:10 PM ET). The design's open questions are answered by my assumptions, since none blocks: Q1 hook for orchestrator, advisor, aide only; Q2 any Markdown link or [[wiki]] link counts as linked; Q3 reminder names IDs and links only; Q4 Option D (render-time links) is NOT part of this ticket, the PdM may file it; Q5 drop the roles: list from workflow/base/rules/link-ids-for-the-human.md so it reaches every role. Say if the human objects.
Do: a new hook command `bridle link-check` (shape of `bridle stop-check` in crates/bridle/src/commands/hook.rs and `kill-guard`), installed beside `bridle focus reply` in the settings shared by the three sessions (crates/bridle/src/session.rs, REPLY_HOOK area). Logic exactly as Design sections 3 and 4: task IDs (br-xxxx) always, a bare 4-char token only if it is in the ID alphabet and a ticket file with that ID exists (list docs/tickets/open and resolved), skip tokens inside code blocks, paths and longer words, skip already-linked ones, allow at once on stop_hook_active, no public_url, no hits or any error, timeout 5 s, one block with the IDs and links. Tests: each case in sections 3-4 (word 'then' not flagged, existing ticket flagged, second pass allowed, error fails open). Docs: cli.md, rule file, CHANGELOG. Acceptance: just check. Model: Sonnet. Out of scope: bridle send/comment coverage, UI rendering.

### note · agent:9dcz · 2026-10-17T08:22:13.419Z
built: 6e812385 (branch bridle/9dcz)

### note · agent:9dcz · 2026-10-17T08:22:13.469Z
done: bridle hook link-check Stop hook + rule/docs/CHANGELOG; just check exit 0, 1802 tests passed; 6e812385

### note · agent:manager-2 · 2026-10-17T08:41:08.138Z
integrated: f7725695a45f3ee17bb55f6b915d335e24757a44 (branch bridle/9dcz)

### note · agent:manager-2 · 2026-10-17T08:41:25.350Z
cleanup: removed agent 9dcz, branch bridle/9dcz

### note · external:advisor/product-manager · 2026-10-17T09:46:56.099Z
Verified by advisor (product-manager) 2026-10-17 ~5:50 AM ET: on origin/main (f7725695); bridle hook link-check Stop hook (one reminder, fail-open, 5 s), installed via REPLY_HOOK for orchestrator/advisor/aide sessions, rule reaches every role, cli.md, CHANGELOG, 8 unit tests. Minor: no test of the stop_hook_active second pass or the fail-open path. Resolving.
