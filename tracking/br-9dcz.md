+++
id = "br-9dcz"
title = "Agents are reminded once to link the ticket and task IDs they show the human (a hook, never a block)"
kind = "feature"
state = "planned"
created_at = "2026-10-10T17:09:36.504Z"
updated_at = "2026-10-17T02:07:32.845467Z"
created_by = "external:advisor/product-manager"
watchers = ["external:advisor/product-manager"]
ticket = "9dcz"
+++

docs/tickets/open/agents-are-reminded-once-to-link-the-ticket-and-task-ids-the-9dcz.md

## Thread

### note · external:advisor/product-manager · 2026-10-17T02:06:33.258Z
The human, 2026-10-16 ~10:10 PM ET, to advisor (product-manager): "Yes I approve the \"cheap batch\" of tickets mentioned earlier".

### note · agent:pm-1 · 2026-10-17T02:07:21.421Z
Brief (pm-1): build Option B of the Design section (the human approved the ticket, 2026-10-16 ~10:10 PM ET). The design's open questions are answered by my assumptions, since none blocks: Q1 hook for orchestrator, advisor, aide only; Q2 any Markdown link or [[wiki]] link counts as linked; Q3 reminder names IDs and links only; Q4 Option D (render-time links) is NOT part of this ticket, the PdM may file it; Q5 drop the roles: list from workflow/base/rules/link-ids-for-the-human.md so it reaches every role. Say if the human objects.
Do: a new hook command `bridle link-check` (shape of `bridle stop-check` in crates/bridle/src/commands/hook.rs and `kill-guard`), installed beside `bridle focus reply` in the settings shared by the three sessions (crates/bridle/src/session.rs, REPLY_HOOK area). Logic exactly as Design sections 3 and 4: task IDs (br-xxxx) always, a bare 4-char token only if it is in the ID alphabet and a ticket file with that ID exists (list docs/tickets/open and resolved), skip tokens inside code blocks, paths and longer words, skip already-linked ones, allow at once on stop_hook_active, no public_url, no hits or any error, timeout 5 s, one block with the IDs and links. Tests: each case in sections 3-4 (word 'then' not flagged, existing ticket flagged, second pass allowed, error fails open). Docs: cli.md, rule file, CHANGELOG. Acceptance: just check. Model: Sonnet. Out of scope: bridle send/comment coverage, UI rendering.
