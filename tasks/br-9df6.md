+++
id = "br-9df6"
title = "Quiet hours: a much more forceful gate, with hard limits (cdez)"
kind = "feature"
state = "integrated"
created_at = "2026-10-01T02:29:11.505Z"
updated_at = "2026-10-01T02:41:23.758688Z"
size = "S"
priority = "high"
branch = "bridle/quiet-gate"
commit = "5a6261b6ab5876056cf48babaf8a9cae5144c141"
summary = "Quiet-hours gate text (crates/bridle/src/focus.rs nudge_text) now gives hard limits: <=3 sentences/60 words, first a nudge, no extra tool calls/research/tickets/planning/threads, defer as 'saved for <end> ET', no follow-ups. Advisor/orchestrator role text point at it; roles-and-config.md and CHANGELOG updated. Locked mode and no-[[focus]] silence unchanged (existing tests). Test asserts limits and end time."
+++

Ticket: docs/tickets/open/quiet-hours-aren-t-quiet-agents-still-talk-too-long-cdez.md (read it); cvaq design in docs/design/agent-host/roles-and-config.md ([[focus]]); code: crates/bridle/src/focus.rs (bridle focus gate, the UserPromptSubmit hook); role text: workflow/base/roles/advisor.md (Style), workflow/base/roles/orchestrator.md ("Quiet hours").

Problem: tonight the gate's soft sentence ("lead with a nudge, keep the answer minimal") was ignored; the advisor and orchestrator still wrote multi-paragraph answers, filed tickets and ran many tool calls during quiet hours.

Goal: in a quiet period the injected context is firm, specific and measurable. Rewrite the gate text (keep it short, imperative, at the top of the injected context) to give HARD limits: at most 3 sentences or 60 words in the reply; the first sentence is the nudge to go back to their real work; no tool calls except the one the human asked for, no research, no ticket filing or planning, no new threads: anything extra is deferred with one line ("saved for <end time> ET") and written down only as a single bridle message to self if truly needed; do not ask follow-up questions unless the human asked for something that cannot proceed. Name the period and its end time. Locked mode keeps its own behaviour; do not change it.
Also update the role text in advisor.md and orchestrator.md to the same limits, one short paragraph each, pointing at the gate as the source.
Keep focus OFF when [[focus]] is not configured: the gate injects nothing then (test that), so projects without [[focus]] see no change. Nothing in the daemon start-up path: this is a session hook only.
Tests: the gate output in a quiet period contains the limits and the end time; no output outside any window / without [[focus]]; locked mode output unchanged (snapshot of the existing test). Docs: roles-and-config.md, CHANGELOG. Acceptance: just check passes.
Model: Sonnet (wording matters; keep it terse). Out of scope: enforcing length by blocking replies, a Stop hook, per-role text variants, daemon changes.

## Thread

### note · agent:pm-1 · 2026-10-01T02:29:11.506Z
priority: normal -> high

### note · agent:quiet-gate · 2026-10-01T02:40:56.881Z
done: quiet-hours gate now injects hard limits (3 sentences/60 words, nudge first, no extra tools/research/tickets, 'saved for <end> ET'); role text, docs, CHANGELOG updated; just check green (947 tests); 53ec4ec

### note · agent:manager-2 · 2026-10-01T02:41:04.730Z
integrated: 5a6261b6ab5876056cf48babaf8a9cae5144c141 (branch bridle/quiet-gate)

### note · agent:manager-2 · 2026-10-01T02:41:23.758Z
cleanup: removed agent quiet-gate, branch bridle/quiet-gate
